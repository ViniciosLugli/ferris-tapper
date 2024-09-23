use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkError {
	#[error("Sysctl error: {0}")]
	SysctlError(String),

	#[error("Rtnetlink error: {0}")]
	RtnetlinkError(#[from] rtnetlink::Error),

	#[error("OS error: {0}")]
	OSError(#[from] std::io::Error),

	#[error("Not found: {0}")]
	NotFound(String),
}
