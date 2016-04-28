use shared_library;

use ffi;

/// Possible errors.
#[derive(Debug)]
pub enum Error {
	/// Open-GL error with the name of the function which caused the error and the error code returned by glGetError().
	Gl(&'static str, ffi::GLenum),
	/// Is used when the return value of a function call indicates an error.
	Fn(&'static str),
	/// Shared library error.
	Sl(shared_library::LoadingError),
	/// Dynamic library error.
	Dl(String),
}
// Function used to check whether an opengl error is present.
pub unsafe fn gl_error(lib_glesv2: &ffi::LibGLESv2, name: &'static str) -> Result<(), Error> {
	let error = (lib_glesv2.glGetError)();
	if error != 0 {
		Err(Error::Gl(name, error))
	} else {
		Ok(())
	}
}

