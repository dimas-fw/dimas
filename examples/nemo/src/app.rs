//! The UI definition for netmon, a network monitoring toolset based on DiMAS
//! Copyright © 2023 Stephan Kunz

use makepad_widgets::file_tree::*;
use makepad_widgets::*;

use crate::network_tree::{NetworkSystem, NetworkTreeAction};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    App = {{App}} {
        // creation/definition of the User Interface
        ui: <Window> {
            window: { inner_size: vec2(1024, 768) }
//            caption_bar = { draw_bg: { color: #86608e44 }, visible: false, caption_label = { label = { text: "NeMo a Network Monitor" } } }
//            window_menu = {
//                main = Main { items: [app, file] }
//
//                app = Sub { name: "Network Monitor", items: [about, line, quit] }
//                about = Item { name: "Network Monitor", enabled: false }
//                quit = Item { name: "Quit Network Monitor", key: KeyQ }
//
//                file = Sub { name: "File", items: [close_window] }
//                close_window = Item { name: "Close Window", enabled: false }
//            }

            show_bg: true
            width: Fill
            height: Fill

            draw_bg: {
                fn pixel(self) -> vec4 {
                    return mix(#10, #3, self.pos.y);
                }
            }

            body = {
                dock = <Dock> {
                    height: Fill
                    width: Fill

                    root = Splitter {
                        axis: Horizontal
                        //align: Weighted(0.2)
                        align: FromA(256.0)
                        a: left_tabs
                        b: split1
                    }

                    split1 = Splitter {
                        axis: Vertical
                        //align: Weighted(0.7)
                        align: FromB(256.0)
                        a: main_tabs
                        b: bottom_tabs
                    }

                    left_tabs = Tabs {
                        tabs: [net_tree]
                        selected: 0
                    }

                    main_tabs = Tabs {
                        tabs: [welcome]
                        selected: 0
                    }

                    bottom_tabs = Tabs {
                        tabs: [log_list]
                        selected: 0
                    }

                    net_tree = Tab {
                        name: "Explore"
                        closable: false
                        kind: NetTree
                    }

                    welcome = Tab {
                        name: "Welcome"
                        closable: true
                        kind: Welcome
                    }

                    log_list = Tab {
                        name: "Log"
                        closable: true
                        kind: LogList
                    }

                    NetTree = <FileTree> {}

                    Welcome = <RectView> {
                        draw_bg: { color: #6c308244 }   // Eminence
                        //draw_bg: { color: #86608e44 }   // Pomp & Power
                        width: Fill
                        height: Fill
                        align: { x: 0.5, y: 0.5 }
                        flow: Down
                        // german
                        <Label> {
                            text: "Willkommen zu NeMo"
                            draw_text: {
                                text_style: {
                                    font_size: 20.0
                                    height_factor: 1.0
                                }
                            }
                        }
                        <Label> {
                            text: "\nDem Netzwerk Monitoring Tool\n\n"
                        }
                        // thai
                        <Label> {
                            text: "ยินดีต้อนรับสู่ NeMo"   // 尼莫
                            draw_text: {
                                text_style: {
                                    font_size: 20.0
                                    height_factor: 1.0
                                }
                            }
                        }
                        <Label> {
                            text: "\nเครื่องมือตรวจสอบเครือข่าย\n\n"
                        }
                        // english
                        <Label> {
                            text: "Welcome to NeMo"
                            draw_text: {
                                text_style: {
                                    font_size: 20.0
                                    height_factor: 1.0
                                }
                            }
                        }
                        <Label> {
                            text: "\nThe network monitoring tool\n\n"
                        }
                        // chinese
                        <Label> {
                            text: "欢迎来到 NeMo"   // 尼莫
                            draw_text: {
                                text_style: {
                                    font_size: 20.0
                                    height_factor: 1.0
                                }
                            }
                        }
                        <Label> {
                            text: "\n网络监控工具"
                        }
                    }

                    LogList = <RectView> {}

                }
            }
        }
    }
}

app_main!(App);

#[derive(Live)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    network: NetworkSystem,
}

impl LiveHook for App {
    fn before_live_design(cx: &mut Cx) {
        crate::makepad_widgets::live_design(cx);

        cx.start_stdin_service();
    }

    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        self.network.init(cx);
    }
}

impl App {}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        // components
        let dock = self.ui.dock(id!(dock));
        //let left_tabs = self.ui.portal_list(id!(left_tabs));
        let net_tree = self.ui.file_tree(id!(net_tree));
        //let main_tabs = self.ui.portal_list(id!(main_tabs));
        //let bottom_tabs = self.ui.portal_list(id!(bottom_tabs));
        let log_list = self.ui.portal_list(id!(log_list));

        // redraw event
        if let Event::Draw(event) = event {
            let cx = &mut Cx2d::new(cx, event);
            //dbg!(&self.network);
            while let Some(next) = self.ui.draw_widget(cx).hook_widget() {
                //dbg!("tree2");
                if let Some(mut net_tree) = net_tree.has_widget(&next).borrow_mut() {
                    //dbg!("tree3");
                    net_tree.set_folder_is_open(cx, live_id!(root).into(), true, Animate::No);
                    self.network
                        .draw_node(cx, live_id!(root).into(), &mut net_tree);
                } else if let Some(mut log_list) = log_list.has_widget(&next).borrow_mut() {
                    log_list.redraw(cx);
                }
            }
            return; // bail out after drawing
        }

        // Network events
        for action in self.network.handle_event(cx, event, &self.ui) {
            match action {
                NetworkTreeAction::Nothing => {
                    dbg!("NetworkTreeAction::None");
                    //net_tree.redraw(cx);
                }
                NetworkTreeAction::RedrawTree => {
                    //dbg!("NetworkTreeAction::RedrawTree");
                    net_tree.redraw(cx);
                }
            }
        }

        // handles also the splitter events
        let actions = self.ui.handle_widget_event(cx, event);

        // any close event on a tab?
        if let Some(tab_id) = dock.clicked_tab_close(&actions) {
            dbg!("close tab");
            dock.close_tab(cx, tab_id);
        }
    }
}
