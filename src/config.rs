use std::ops::Deref;
use std::path::{Path, PathBuf};

use libc;

/// Wrapper for the directory where the libraries are stored. Defaults to /opt/vc/lib
pub struct LibDir(pub PathBuf);
impl Default for LibDir {
	fn default() -> Self {
		LibDir(Path::new("/opt/vc/lib").to_path_buf())
	}
}
impl Deref for LibDir {
	type Target = Path;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// Possible displays. Defaults to Hdmi.
#[derive(Clone, Copy)]
pub enum Display {
	Hdmi,
	Analog,
	Lcd,
}
impl Display {
	pub fn index(&self) -> libc::uint16_t {
		match *self {
			Display::Hdmi => 2,
			Display::Analog => 1,
			Display::Lcd => 0,
		}
	}
}
impl Default for Display {
	fn default() -> Self { Display::Hdmi }
}

/// Wrapper for color bits. Defaults to 8.
#[derive(Copy, Clone)]
pub struct ColorBits(pub u32);
impl Default for ColorBits {
	fn default() -> Self {
		ColorBits(8)
	}
}
impl Deref for ColorBits {
	type Target = u32;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// Wrapper for depth bits. Defaults to 16.
#[derive(Copy, Clone)]
pub struct DepthBits(pub u32);
impl Default for DepthBits {
	fn default() -> Self {
		DepthBits(16)
	}
}
impl Deref for DepthBits {
	type Target = u32;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// Window configuration.
#[derive(Copy, Clone, Default)]
pub struct WindowConfig {
	/// The display to use.
	pub display: Display,
	/// The size of the surface to render to. If none, the size of the display is used.
	pub surface_size: Option<(u32, u32)>,
	/// Number of bits per pixel used for the red channel.
	pub red: ColorBits,
	/// Number of bits per pixel used for the green channel.
	pub green: ColorBits,
	/// Number of bits per pixel used for the blue channel.
	pub blue: ColorBits,
	/// Number of bits per pixel used for the alpha channel.
	pub alpha: Option<ColorBits>,
	/// Number of bits per pixel used for the depth buffer.
	pub depth: Option<DepthBits>,
}


