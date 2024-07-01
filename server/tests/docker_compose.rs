use std::error;
use std::fmt;
use std::{
	collections::HashMap,
	path::PathBuf,
	process::{Command, Output},
};

#[derive(Debug)]
pub enum DockerComposeError {
	Spawn(std::io::Error),
	Wait(std::io::Error),
	ExitStatus(std::process::ExitStatus),
}

impl fmt::Display for DockerComposeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

impl error::Error for DockerComposeError {}

#[derive(Debug)]
pub struct DockerComposeConfig {
	pub compose_file: PathBuf,
	pub env: HashMap<String, String>,
	pub project_name: String,
}

pub struct DockerCompose {
	config: DockerComposeConfig,
}

impl DockerCompose {
	fn base_cmd(config: &DockerComposeConfig) -> Command {
		let mut cmd = Command::new("docker");
		cmd.arg("compose");
		cmd.arg("--file");
		cmd.arg(&config.compose_file);
		cmd.arg("--project-name");
		cmd.arg(&config.project_name);
		cmd.envs(&config.env);
		cmd
	}

	fn up_cmd(config: &DockerComposeConfig) -> Command {
		let mut cmd = DockerCompose::base_cmd(config);
		cmd.arg("up");
		cmd.arg("--wait");
		cmd.arg("--timeout");
		cmd.arg("5");
		cmd
	}

	fn down_cmd(config: &DockerComposeConfig) -> Command {
		let mut cmd = DockerCompose::base_cmd(config);
		cmd.arg("down");
		cmd.arg("--volumes");
		cmd.arg("--rmi");
		cmd.arg("local");
		cmd.arg("--timeout");
		cmd.arg("5");
		cmd
	}

	fn restart_cmd(config: &DockerComposeConfig, service: &str) -> Command {
		let mut cmd = DockerCompose::base_cmd(config);
		cmd.arg("restart");
		cmd.arg("--timeout");
		cmd.arg("10");
		cmd.arg(service);
		cmd
	}

	fn logs_cmd(config: &DockerComposeConfig) -> Command {
		let mut cmd = DockerCompose::base_cmd(config);
		cmd.arg("logs");
		cmd
	}

	fn run(mut cmd: Command) -> Result<(), DockerComposeError> {
		print!("Running command {:?}", cmd);
		cmd.spawn()
			.map_err(DockerComposeError::Spawn)
			.and_then(|mut h| h.wait().map_err(DockerComposeError::Wait))
			.and_then(|status| {
				if status.success() {
					Ok(())
				} else {
					Err(DockerComposeError::ExitStatus(status))
				}
			})
	}

	#[allow(dead_code)]
	fn run_with_output(mut cmd: Command) -> Result<Output, DockerComposeError> {
		print!("Running command {:?}", cmd);
		cmd.output().map_err(DockerComposeError::Spawn)
	}

	#[allow(dead_code)]
	pub fn up_with_output(
		config: DockerComposeConfig,
	) -> Result<(DockerCompose, Output), DockerComposeError> {
		print!("Starting docker compose with {:?}", config);
		let cmd = DockerCompose::up_cmd(&config);
		DockerCompose::run_with_output(cmd).map(|o| (DockerCompose { config }, o))
	}

	pub fn up(config: DockerComposeConfig) -> Result<DockerCompose, DockerComposeError> {
		print!("Starting docker compose with {:?}", config);
		let cmd = DockerCompose::up_cmd(&config);
		DockerCompose::run(cmd).map(|e| DockerCompose { config })
	}

	pub fn restart(&self, service: &str) -> Result<(), DockerComposeError> {
		print!("Restarting docker compose container {} with {:?}", service, self.config);
		let cmd = DockerCompose::restart_cmd(&self.config, service);
		DockerCompose::run(cmd)
	}

	fn down(&self) -> Result<(), DockerComposeError> {
		let cmd = DockerCompose::down_cmd(&self.config);
		DockerCompose::run(cmd)
	}

	fn logs(&self) -> Result<(), DockerComposeError> {
		let cmd = DockerCompose::logs_cmd(&self.config);
		DockerCompose::run(cmd)
	}
}

impl Drop for DockerCompose {
	fn drop(&mut self) {
		if std::thread::panicking() {
			self.logs().expect("Could not run docker compose logs");
		}
		eprintln!("Dropping docker compose");
		self.down().expect("Could not run docker compose down");
	}
}
