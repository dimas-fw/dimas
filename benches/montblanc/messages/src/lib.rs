// Copyright © 2024 Stephan Kunz

//! all the messages needed for montblanc benchmark
//! based on ros2 messages, see
//! - <https://github.com/ros2/rcl_interfaces>
//! - <https://github.com/ros2/common_interfaces>
//! should be modernized and moved into a separate crate

use std::fmt::Display;

use bitcode::{Decode, Encode};
use chrono::prelude::*;
use rand::distributions::Alphanumeric;
use rand::{random, Rng};

/// function creates a random String of given length
pub fn random_string(length: usize) -> String {
	rand::thread_rng()
		.sample_iter(Alphanumeric)
		.take(length)
		.map(char::from)
		.collect()
}

/// Float64 message
#[derive(Debug, Encode, Decode)]
pub struct Float64 {
	/// data
	pub data: f64,
}

impl Float64 {
	/// provides some random `Float64` data
	#[must_use]
	pub fn random() -> Self {
		Self {
			data: 1_000_000_000.0 * random::<f64>(),
		}
	}
}

impl Display for Float64 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.data)
	}
}

/// Float32 message
#[derive(Debug, Encode, Decode)]
pub struct Float32 {
	/// data
	pub data: f32,
}

impl Float32 {
	/// provides some random `Float32` data
	#[must_use]
	pub fn random() -> Self {
		Self {
			data: 1_000_000.0 * random::<f32>(),
		}
	}
}

impl Display for Float32 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.data)
	}
}

/// Int64 message
#[derive(Debug, Encode, Decode)]
pub struct Int64 {
	/// data
	pub data: i64,
}

impl Int64 {
	/// provides some random `Int64` data
	#[must_use]
	pub fn random() -> Self {
		Self { data: random() }
	}
}

impl Display for Int64 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.data)
	}
}

/// Int32 message
#[derive(Debug, Encode, Decode)]
pub struct Int32 {
	/// data
	pub data: i32,
}

impl Int32 {
	/// provides some random `Int32` data
	#[must_use]
	pub fn random() -> Self {
		Self { data: random() }
	}
}

impl Display for Int32 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.data)
	}
}

/// String message
#[derive(Debug, Encode, Decode)]
pub struct StringMsg {
	/// data
	pub data: String,
}

impl StringMsg {
	/// provides some random `StringMsg` data
	#[must_use]
	pub fn random() -> Self {
		Self {
			data: random_string(64),
		}
	}
}

impl Display for StringMsg {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.data)
	}
}

/// Timestamp message
/// Representing the elapsed seconds since 1.1.1970 00:00:00.000. Negative values are before that date.
#[derive(Debug, Encode, Decode)]
pub struct Timestamp {
	/// seconds, valid over all i64 values
	pub sec: i64,
	/// The nanoseconds component if available, valid in the range [0, 1e9)
	pub nanosec: u32,
}

impl Timestamp {
	/// Creates a `Timestamp` of now
	#[must_use]
	pub fn now() -> Self {
		let now = Utc::now();
		Self {
			sec: now.timestamp(),
			nanosec: now.nanosecond(),
		}
	}
}

impl Display for Timestamp {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "timestamp: {}.{}", self.sec, self.nanosec)
	}
}

/// Header message
/// Standard metadata for higher-level stamped data types.
/// This is generally used to communicate timestamped data
/// in a particular coordinate frame.
#[derive(Debug, Encode, Decode)]
pub struct Header {
	/// Timestamp of message creation
	pub timestamp: Timestamp,
	/// the frame id
	pub frame_id: String,
}

impl Header {
	/// provides a new `Header`
	#[must_use]
	pub fn new() -> Self {
		Self {
			timestamp: Timestamp::now(),
			frame_id: "Test".into(),
		}
	}
}

impl Default for Header {
	#[must_use]
	fn default() -> Self {
		Self {
			timestamp: Timestamp::now(),
			frame_id: "Default".into(),
		}
	}
}

impl Display for Header {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}, id: {}", self.timestamp, self.frame_id)
	}
}

/// Point message
/// Contains the position of a point in free space
#[derive(Debug, Encode, Decode)]
pub struct Point {
	/// x value
	pub x: f64,
	/// y value
	pub y: f64,
	/// z value
	pub z: f64,
}

impl Point {
	/// provides some random `Point` data
	#[must_use]
	pub fn random() -> Self {
		Self {
			x: random(),
			y: random(),
			z: random(),
		}
	}
}

impl Display for Point {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
	}
}

/// Quaternion message
/// Represents an orientation in free space in quaternion form
#[derive(Debug, Encode, Decode)]
pub struct Quaternion {
	/// x value
	pub x: f64,
	/// y value
	pub y: f64,
	/// z value
	pub z: f64,
	/// theta value
	pub w: f64,
}

impl Quaternion {
	/// provides some random `Quaternion` data
	#[must_use]
	pub fn random() -> Self {
		Self {
			x: random(),
			y: random(),
			z: random(),
			w: random(),
		}
	}
}

impl Display for Quaternion {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "[{}, {}, {}, {}]", self.x, self.y, self.z, self.w)
	}
}

/// 3-Dimensional Vector message
/// Represents a vector in free space
/// This is semantically different than a point.
/// A vector is always anchored at the origin.
/// When a transform is applied to a vector, only the rotational component is applied.
#[derive(Debug, Encode, Decode)]
pub struct Vector3 {
	/// x value
	pub x: f64,
	/// y value
	pub y: f64,
	/// z value
	pub z: f64,
}

impl Vector3 {
	/// provides some random `Vector3` data
	#[must_use]
	pub fn random() -> Self {
		Self {
			x: random(),
			y: random(),
			z: random(),
		}
	}
}

impl Display for Vector3 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
	}
}

/// 3D-Vector with Header
/// Represents a Vector3 with reference coordinate frame and timestamp
/// Note that this follows vector semantics with it always anchored at the origin,
/// so the rotational elements of a transform are the only parts applied when transforming.
#[derive(Debug, Encode, Decode)]
pub struct Vector3Stamped {
	/// Timestamp and frame id
	pub header: Header,
	/// the Vector3 data
	pub vector: Vector3,
}

impl Vector3Stamped {
	/// provides some random `Vector3Stamped` data
	#[must_use]
	pub fn random() -> Self {
		Self {
			header: Header::new(),
			vector: Vector3::random(),
		}
	}
}

impl Display for Vector3Stamped {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}, {}", self.header, self.vector)
	}
}

/// Pose message
/// Representation of pose in free space, composed of position and orientation
#[derive(Debug, Encode, Decode)]
pub struct Pose {
	/// Position is a Point in free space
	pub position: Point,
	/// Orientation is a Quaternion
	pub orientation: Quaternion,
}

impl Pose {
	/// provides some random `Pose` data
	#[must_use]
	pub fn random() -> Self {
		Self {
			position: Point::random(),
			orientation: Quaternion::random(),
		}
	}
}

impl Display for Pose {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"position: {} orientation: {}",
			self.position, self.orientation
		)
	}
}

/// Twist message
/// Expresses velocity in free space broken into its linear and angular parts
#[derive(Debug, Encode, Decode)]
pub struct Twist {
	/// Linear velocity
	pub linear: Vector3,
	/// Angular velocity
	pub angular: Vector3,
}

impl Twist {
	/// provides some random `Twist` data
	#[must_use]
	pub fn random() -> Self {
		Self {
			linear: Vector3::random(),
			angular: Vector3::random(),
		}
	}
}

impl Display for Twist {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "linear: {}, angular: {}", self.linear, self.angular)
	}
}

/// Twist with Covariance message
#[derive(Debug, Encode, Decode)]
pub struct TwistWithCovariance {
	/// Twist
	pub twist: Twist,
	/// Row-major representation of the 6x6 covariance matrix
	/// The orientation parameters use a fixed-axis representation.
	/// In order, the parameters are:
	/// (x, y, z, rotation about X axis, rotation about Y axis, rotation about Z axis)
	pub covariance: [f64; 36],
}

impl TwistWithCovariance {
	/// provides some random `TwistWithCovariance` data
	#[must_use]
	pub fn random() -> Self {
		Self {
			twist: Twist::random(),
			covariance: [0.0f64; 36],
		}
	}
}

impl Display for TwistWithCovariance {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}, covariance: {:?}",
			self.twist, self.covariance
		)
	}
}

/// Twist with Covariance and Header message
#[derive(Debug, Encode, Decode)]
pub struct TwistWithCovarianceStamped {
	/// Timestamp and frame id
	pub header: Header,
	/// Twist with Covariance
	pub twist: TwistWithCovariance,
}

impl TwistWithCovarianceStamped {
	/// provides some random `TwistWithCovarianceStamped` data
	#[must_use]
	pub fn random() -> Self {
		Self {
			header: Header::new(),
			twist: TwistWithCovariance::random(),
		}
	}
}

impl Display for TwistWithCovarianceStamped {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}, {}", self.header, self.twist)
	}
}

/// Wrench message - represents force in free space, separated into its linear and angular parts
#[derive(Debug, Encode, Decode)]
pub struct Wrench {
	/// Linear part is force
	pub force: Vector3,
	/// Angular part is torque
	pub torque: Vector3,
}

impl Wrench {
	/// provides some random `Wrench` data
	#[must_use]
	pub fn random() -> Self {
		Self {
			force: Vector3::random(),
			torque: Vector3::random(),
		}
	}
}

impl Display for Wrench {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "force: {}, torque: {}", self.force, self.torque)
	}
}

/// Wrench with Header message
#[derive(Debug, Encode, Decode)]
pub struct WrenchStamped {
	/// Timestamp and frame id
	pub header: Header,
	/// Wrench
	pub wrench: Wrench,
}

impl WrenchStamped {
	/// provides some random `WrenchStamped` data
	#[must_use]
	pub fn random() -> Self {
		Self {
			header: Header::new(),
			wrench: Wrench::random(),
		}
	}
}

impl Display for WrenchStamped {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}, {}", self.header, self.wrench)
	}
}

/// Image message
/// Contains an uncompressed image
/// (0, 0) is at top-left corner of image
/// +x should point to the right in the image
/// +y should point down in the image
/// +z should point into to plane of the image
#[derive(Debug, Encode, Decode)]
pub struct Image {
	/// Header timestamp should be acquisition time of image
	/// Header frame_id should be optical frame of camera
	/// origin of frame should be optical center of cameara
	/// If the frame_id here and the frame_id of the CameraInfo
	/// message associated with the image conflict the behavior is undefined
	pub header: Header,
	/// Image height, that is, number of rows
	pub height: u32,
	/// Image width, that is, number of columns
	pub width: u32,
	/// Encoding of pixels -- channel meaning, ordering, size
	/// see: <https://github.com/ros2/common_interfaces/blob/rolling/sensor_msgs/include/sensor_msgs/image_encodings.hpp>
	pub encodings: String,
	/// Is this data bigendian?
	pub is_bigendian: bool,
	/// Full row length in bytes
	pub step: u32,
	/// Actual matrix data, size is (step * height). Width is only giving num of pixels not the bytes/pixel step = width * bytes/pixel
	pub data: Vec<u8>,
}

impl Image {
	/// provides some random `Image` data
	#[must_use]
	pub fn random() -> Self {
		let number: u32 = random();
		let header = Header {
			timestamp: Timestamp::now(),
			frame_id: "Image ".to_string() + &number.to_string(),
		};
		let height = random();
		let width = random();
		let encodings = random_string(32);
		let is_bigendian = random();
		let step = 2 * width;
		let data = Vec::with_capacity((step * height) as usize);
		Self {
			header,
			height,
			width,
			encodings,
			is_bigendian,
			step,
			data,
		}
	}
}

impl Display for Image {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}, size: {}x{}",
			self.header, self.width, self.height
		)
	}
}

/// Laser Scan message
/// Single scan from a planar laser range-finder
/// If you have another ranging device with different behavior (e.g. a sonar
/// array), please find or create a different message, since applications
/// will make fairly laser-specific assumptions about this data
#[derive(Debug, Encode, Decode)]
pub struct LaserScan {
	/// timestamp in the header is the acquisition time of
	/// the first ray in the scan.
	/// In frame frame_id, angles are measured around
	/// the positive Z axis (counterclockwise, if Z is up)
	/// with zero angle being forward along the x axis
	pub header: Header,
	/// Start angle of the scan in rad
	pub angle_min: f32,
	/// End angle of the scan in rad
	pub angle_max: f32,
	/// Angular distance between measurements in rad
	pub angle_increment: f32,
	/// Time between measurements in seconds - if your scanner
	/// is moving, this will be used in interpolating position
	/// of 3d points
	pub time_increment: f32,
	/// Time between scans in seconds
	pub scan_time: f32,
	/// Minimum range value in m
	pub range_min: f32,
	/// Maximum range value in m
	pub range_max: f32,
	/// Range data in m (Note: values < range_min or > range_max should be discarded)
	pub ranges: Vec<f32>,
	/// Intensity data in device-specific units.
	/// If your device does not provide intensities, please leave the array empty.
	pub intensities: Vec<f32>,
}

impl LaserScan {
	/// provides some random `LaserScan` data
	#[must_use]
	pub fn random() -> Self {
		let number: u32 = random();
		let header = Header {
			timestamp: Timestamp::now(),
			frame_id: "Scan ".to_string() + &number.to_string(),
		};
		Self {
			header,
			angle_min: 0.0,
			angle_max: 0.0,
			angle_increment: 0.0,
			time_increment: 0.0,
			scan_time: 0.0,
			range_min: 0.0,
			range_max: 0.0,
			ranges: Vec::with_capacity(360),
			intensities: Vec::with_capacity(360),
		}
	}
}

impl Display for LaserScan {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.header)
	}
}

/// Data Type message
/// Definitions for the type of data used in `PointField`
#[derive(Debug, Encode, Decode)]
pub enum DataType {
	/// Int8 type
	Int8,
	/// Unsigned Int8 type
	UInt8,
	/// Int16 type
	Int16,
	/// Unsigned Int16 type
	UInt16,
	/// Int32 type
	Int32,
	/// Unsigned Int32 type
	UInt32,
	/// Float32 type
	Float32,
	/// Float64 type
	Float64,
}

/// Point Field message
/// Holds the description of one point entry in the `PointCloud2` message format
/// Common Point Field names are x, y, z, intensity, rgb, rgba
#[derive(Debug, Encode, Decode)]
pub struct PointField {
	/// Name of field
	pub name: String,
	/// Offset from start of point struct
	pub offset: u32,
	/// Data type enumeration, see `DataType`
	pub datatype: DataType,
	/// How many elements in the field
	pub count: u32,
}

/// Point Cloud message
/// This message holds a collection of N-dimensional points, which may
/// contain additional information such as normals, intensity, etc. The
/// point data is stored as a binary blob, its layout described by the
/// contents of the "fields" array.
/// The point cloud data may be organized 2d (image-like) or 1d (unordered).
/// Point clouds organized as 2d images may be produced by camera depth sensors
/// such as stereo or time-of-flight.
#[derive(Debug, Encode, Decode)]
pub struct PointCloud2 {
	/// Time of sensor data acquisition, and the coordinate frame ID (for 3d points)
	pub header: Header,
	/// Height of the point cloud. If the cloud is unordered, height is 1.
	pub height: u32,
	/// Width of the point cloud. If the cloud is unordered, width is the length of the point cloud.
	pub width: u32,
	/// Describes the channels and their layout in the binary data blob
	pub fields: Vec<PointField>,
	/// Is this data bigendian?
	pub is_bigendian: bool,
	/// Length of a point in bytes
	pub point_step: u32,
	/// Length of a row in bytes
	pub row_step: u32,
	/// Size is (row_step*height)
	pub data: Vec<u8>,
	/// True if there are no invalid points
	pub is_dense: bool,
}

impl PointCloud2 {
	/// provides some random `PointCloud2` data
	#[must_use]
	pub fn random() -> Self {
		Self {
			header: Header::new(),
			height: 0,
			width: 0,
			fields: Vec::new(),
			is_bigendian: true,
			point_step: 4,
			row_step: 4,
			data: Vec::new(),
			is_dense: true,
		}
	}
}

impl Display for PointCloud2 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}, size: {}x{}",
			self.header, self.width, self.height
		)
	}
}
