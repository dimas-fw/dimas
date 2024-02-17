#![allow(dead_code)]
// https://github.com/ros2/rcl_interfaces
// https://github.com/ros2/

//! all the messages needed for montblanc benchmark
//!

use bitcode::{Decode, Encode};

/// Float64 message
#[derive(Encode, Decode)]
pub struct Float64 {
	/// data
	pub data: f64,
}

/// Float32 message
#[derive(Encode, Decode)]
pub struct Float32 {
	/// data
	pub data: f32,
}

/// Int64 message
#[derive(Encode, Decode)]
pub struct Int64 {
	/// data
	pub data: i64,
}

/// Int32 message
#[derive(Encode, Decode)]
pub struct Int32 {
	/// data
	pub data: i32,
}

/// String message
#[derive(Encode, Decode)]
pub struct StringMsg {
	/// data
	pub data: String,
}

/// Timestamp message
#[derive(Encode, Decode)]
pub struct Timestamp {
	pub sec: i32,
	pub nanosec: u32,
}

/// Header message
#[derive(Encode, Decode)]
pub struct Header {
	pub timestamp: Timestamp,
	pub frame_id: String,
}

/// Point message
#[derive(Encode, Decode)]
pub struct Point {
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

/// Quaternion message
#[derive(Encode, Decode)]
pub struct Quaternion {
	pub x: f64,
	pub y: f64,
	pub z: f64,
	pub w: f64,
}

/// 3-Dimenional Vector message
#[derive(Encode, Decode)]
pub struct Vector3 {
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

/// 3D-Vector with Header
#[derive(Encode, Decode)]
pub struct Vector3Stamped {
	pub header: Header,
	pub vectro: Vector3,
}

/// Pose message
#[derive(Encode, Decode)]
pub struct Pose {
	pub position: Point,
	pub orientation: Quaternion,
}

/// Twist message
#[derive(Encode, Decode)]
pub struct Twist {
	pub linear: Vector3,
	pub angular: Vector3,
}

/// Covariance message
#[derive(Encode, Decode)]
pub struct Covariance {
	//pub values: f64[36],
}

/// Twist with Covariance message
#[derive(Encode, Decode)]
pub struct TwistWithCovariance {
	pub twist: Twist,
	pub covariance: Covariance,
}

/// Twist with Covariance and Header message
#[derive(Encode, Decode)]
pub struct TwistWithCovarianceStamped {
	pub header: Header,
	pub twist: TwistWithCovariance,
}

/// Wrench message
#[derive(Encode, Decode)]
pub struct Wrench {
	pub force: Vector3,
	pub torque: Vector3,
}

/// Wrench with Header message
#[derive(Encode, Decode)]
pub struct WrenchStamped {
	pub header: Header,
	pub wrench: Wrench,
}

/// Image message
#[derive(Encode, Decode)]
pub struct Image {
	pub header: Header,
	pub height: u32,
	pub width: u32,
	//pub encoding: String,
	pub is_bigendian: bool,
	pub step: u32,
	//pub data: u8[],   // size is height*step (width is only giving num of pixels not the bytes/pixel step = width * bytes/pixel)
}

/// Laser Scan message
#[derive(Encode, Decode)]
pub struct LaserScan {
	pub header: Header,
	pub angle_min: f32,
	pub angle_max: f32,
	pub angle_increment: f32,
	pub time_increment: f32,
	pub scan_time: f32,
	pub range_min: f32,
	pub range_max: f32,
	//pub ranges: f32[],
	//pub intensities: f32[],
}

/// Data Type message
#[derive(Encode, Decode)]
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

/// Point Filed message
#[derive(Encode, Decode)]
pub struct PointField {
	pub name: String,
	pub offset: u32,
	pub datatype: DataType,
	pub count: u32,
}

/// Point Cloud message
#[derive(Encode, Decode)]
pub struct PointCloud2 {
	pub header: Header,
	pub height: u32, //
	pub width: u32,  //
	//pub fields: PointField[],       //
	pub is_bigendian: bool, // Is this data bigendian?
	pub point_step: u32,    // Length of a point in bytes
	pub row_step: u32,      // Length of a row in bytes
	//pub data: u8[],                 // Size is (row_step*height)
	pub is_dense: bool, // True if there are no invalid points
}
