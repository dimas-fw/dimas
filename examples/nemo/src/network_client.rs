//! Copyright © 2023 Stephan Kunz

use std::{
	sync::{
		mpsc::{self, *},
		Arc, RwLock,
	},
	time::Duration,
};

use makepad_widgets::*;
use zenoh::{prelude::r#async::*, subscriber::Subscriber};

use crate::network_protocol::*;

#[derive(Debug, Default, Clone)]
pub enum NetworkResponse {
	#[default]
	DataReceived,
}

#[derive(Debug, Default, Clone)]
pub enum NetworkClientAction {
	#[default]
	Nothing,
	TreeChanged(NetworkTreeData),
}

#[derive(Debug)]
pub struct NetworkClient {
	key_expr: String,
	session: Arc<Session>,
	liveliness_subscriber: Option<Arc<Subscriber<'static, ()>>>,
	action_receiver: Option<Receiver<NetworkResponse>>,
	device_list: Arc<RwLock<NetworkDeviceList>>,
	tree_data: Arc<RwLock<NetworkTreeData>>,
}

impl Default for NetworkClient {
	fn default() -> Self {
		tokio::task::block_in_place(move || {
			tokio::runtime::Handle::current().block_on(async move {
				let session = zenoh::open(config::peer()).res().await.unwrap().into_arc();
				Self {
					key_expr: String::from("nemo/**"),
					session,
					liveliness_subscriber: None,
					action_receiver: None,
					device_list: Arc::new(RwLock::new(NetworkDeviceList::default())),
					tree_data: Arc::new(RwLock::new(NetworkTreeData::default())),
				}
			})
		})
	}
}

impl NetworkClient {
	fn tree_data(&self) -> NetworkTreeData {
		self.tree_data.read().unwrap().clone()
	}

	pub fn init(&mut self, _cx: &mut Cx) {
		tokio::task::block_in_place(move || {
			tokio::runtime::Handle::current().block_on(async move {
				// create root for network tree
				self.tree_data = Arc::new(RwLock::new(NetworkTreeData {
					root: NetworkTreeNode::Root {
						entries: Vec::new(),
					},
					changed: true,
				}));

				let (action_sender, action_receiver) = mpsc::channel();
				self.action_receiver = Some(action_receiver);

				self.key_expr = String::from("nemo/**");
				self.session = zenoh::open(config::peer()).res().await.unwrap().into_arc();

				// add a liveliness subscriber
				let device_list = self.device_list.clone();
				let tree_data = self.tree_data.clone();
				let sender = action_sender.clone();
				self.liveliness_subscriber = Some(Arc::new(
					self.session
						.liveliness()
						.declare_subscriber(&self.key_expr)
						.callback(move |sample: Sample| {
							//dbg!("got data");
							//dbg!(&sample);
							match sample.kind {
								SampleKind::Put => {
									NetworkClient::add(
										device_list.clone(),
										tree_data.clone(),
										sample,
									);
								}
								SampleKind::Delete => {
									NetworkClient::remove(
										device_list.clone(),
										tree_data.clone(),
										sample,
									);
								}
							}
							sender.send(NetworkResponse::DataReceived).unwrap();
							Signal::new().set();
						})
						.res()
						.await
						.unwrap(),
				));

				// the initial liveliness query
				let replies = self
					.session
					.liveliness()
					.get(&self.key_expr)
					//.timeout(Duration::from_millis(500))
					.res()
					.await
					.unwrap();

				while let Ok(reply) = replies.recv_async().await {
					match reply.sample {
						Ok(sample) => {
							//dbg!(&sample);
							Self::add(self.device_list.clone(), self.tree_data.clone(), sample);
							action_sender.send(NetworkResponse::DataReceived).unwrap();
							Signal::new().set();
						}
						Err(err) => println!(
							">> Received (ERROR: '{}')",
							String::try_from(&err).unwrap_or("".to_string())
						),
					}
				}
			})
		});
	}

	pub fn add(
		list: Arc<RwLock<NetworkDeviceList>>,
		tree_data: Arc<RwLock<NetworkTreeData>>,
		sample: Sample,
	) {
		//dbg!(&sample);
		tokio::task::block_in_place(move || {
			tokio::runtime::Handle::current().block_on(async move {
				let session = zenoh::open(config::peer()).res().await.unwrap().into_arc();

				//first fetch necessary data, than add to list and last insert into tree
				let uuid = sample.key_expr.to_string();
				let key_expr = uuid.clone() + "/network";
				//let session = session.clone();
				//dbg!(&key_expr);
				let replies = session
					.get(&key_expr)
					// ensure to get more than one interface from a host
					.consolidation(ConsolidationMode::None)
					.timeout(Duration::from_millis(500))
					.res()
					.await
					.unwrap();
				//dbg!(&replies);

				while let Ok(reply) = replies.recv_async().await {
					//dbg!(&reply);
					match reply.sample {
						Ok(sample) => {
							//dbg!(&sample);
							let device: NetworkDeviceData =
								serde_json::from_str(sample.value.to_string().as_str()).unwrap();
							//dbg!(&device);
							let entry = NetworkTreeEntry {
								name: device.name.clone() + "/" + &device.ifname,
								node: NetworkTreeNode::Host {
									name: device.name.clone(),
									ip: "".to_string(),
								},
							};
							NetworkClient::add_to_tree(tree_data.clone(), entry)
						}
						Err(err) => println!(
							">> No data (ERROR: '{}')",
							String::try_from(&err).unwrap_or("".to_string())
						),
					}
				}

				let list = &mut list.write().unwrap();
				let device = NetworkDevice {
					uuid: uuid.clone(),
					..Default::default()
				};
				list.nodes.insert(uuid, device);
				//dbg!(&list);

				//dbg!(&tree_data);
			})
		})
	}

	pub fn remove(
		list: Arc<RwLock<NetworkDeviceList>>,
		tree_data: Arc<RwLock<NetworkTreeData>>,
		sample: Sample,
	) {
		//dbg!(&sample);
		// first remove from tree, than remove from list

		NetworkClient::remove_from_tree(tree_data);
		//dbg!(&tree_data);

		let list = &mut list.write().unwrap();
		let uuid = sample.key_expr.to_string();
		list.nodes.remove(&uuid);

		//dbg!(&list);
	}

	pub fn add_to_tree(tree_data: Arc<RwLock<NetworkTreeData>>, entry: NetworkTreeEntry) {
		//dbg!(tree_data);
		let tree_data = &mut tree_data.write().unwrap();
		match &mut tree_data.root {
			NetworkTreeNode::Root { entries } => {
				entries.push(entry);
				tree_data.changed = true;
			}
			_ => todo!(),
		}
		//dbg!(tree_data);
	}

	pub fn remove_from_tree(tree_data: Arc<RwLock<NetworkTreeData>>) {
		let _tree_data = &mut tree_data.write().unwrap();
		//todo!();
		//match &mut tree_data.root {
		//    NetworkTreeNode::Root{entries} => todo!(),
		//    _ => todo!(),
		//}
		//dbg!(tree_data);
	}

	pub fn handle_event(&mut self, cx: &mut Cx, event: &Event) -> Vec<NetworkClientAction> {
		let mut actions = Vec::new();
		self.handle_event_with(cx, event, &mut |_, action| actions.push(action));
		actions
	}

	fn handle_event_with(
		&mut self,
		cx: &mut Cx,
		event: &Event,
		dispatch_action: &mut dyn FnMut(&mut Cx, NetworkClientAction),
	) {
		//dbg!("NetworkClient::handle_event_with");
		//dbg!("NetworkClient::handle_event_with {}", &event);
		let receiver = self.action_receiver.as_ref().unwrap();
		match event {
			Event::Signal => loop {
				match receiver.try_recv() {
					Ok(_response) => {
						dispatch_action(cx, NetworkClientAction::TreeChanged(self.tree_data()))
					}
					Err(TryRecvError::Empty) => break,
					_ => panic!(),
				}
			},
			Event::NextFrame(_) => {
				if self.tree_data.read().unwrap().changed {
					dispatch_action(cx, NetworkClientAction::TreeChanged(self.tree_data()));
					self.tree_data.write().unwrap().changed = false;
				}
			}
			_ => {}
		}
		//if self.tree_data.read().unwrap().changed {
		//    dispatch_action(cx, NetworkClientAction::TreeChanged(self.tree_data()));
		//    self.tree_data.write().unwrap().changed = false;
		//}
	}
}
