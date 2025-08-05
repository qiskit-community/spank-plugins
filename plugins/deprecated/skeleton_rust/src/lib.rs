// This code is part of Qiskit.
//
// (C) Copyright IBM 2025
//
// This program and the accompanying materials are made available under the
// terms of the GNU General Public License version 3, as published by the
// Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <[https://www.gnu.org/licenses/gpl-3.0.txt]
//
use eyre::WrapErr;
use slurm_spank::{Context, Plugin, SpankHandle, SpankOption, SLURM_VERSION_NUMBER, SPANK_PLUGIN};
use tracing::info;

use std::error::Error;
use std::process;

// All spank plugins must define this macro for the Slurm plugin loader.
SPANK_PLUGIN!(b"spank_rust_skeleton", SLURM_VERSION_NUMBER, SpankSkeleton);

#[derive(Default)]
struct SpankSkeleton {
    value: Option<String>,
}

/// Log entering function
macro_rules! enter {
    () => {
        info!("PID = {}, UID = {}", process::id(), unsafe {
            libc::getuid()
        });
    };
}

/// Dump Spank context
macro_rules! dump_context {
    ($spank:expr) => {
        if let Ok(result) = $spank.job_id() {
            info!("S_JOB_ID = {}", result);
        } else {
            info!("S_JOB_ID =");
        }
        if let Ok(result) = $spank.job_stepid() {
            info!("S_JOB_STEPID = {:x}", result);
        } else {
            info!("S_JOB_STEPID =");
        }
        info!("S_JOB_ARGV = {:#?}", $spank.job_argv().unwrap_or(vec!()));
        info!(
            "S_PLUGIN_ARGV = {:#?}",
            $spank.plugin_argv().unwrap_or(vec!())
        );
    };
}

unsafe impl Plugin for SpankSkeleton {
    /// Called just after plugins are loaded.
    ///
    /// In remote context, this is just after job step is initialized. This
    /// function is called before any plugin option processing.
    fn init(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        enter!();
        if spank.context()? == Context::Remote {
            dump_context!(spank);
        }
        // Register the --rust-ske-option=value option
        match spank.context()? {
            Context::Local | Context::Remote | Context::Allocator => {
                spank
                    .register_option(
                        SpankOption::new("rust-ske-option")
                            .takes_value("value")
                            .usage("Option for spank-rust-skeleton."),
                    )
                    .wrap_err("Failed to register spank-rust-skeleton option")?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Called at the same time as the job prolog.
    ///
    /// If this function returns an error and the SPANK plugin that contains it
    /// is required in the plugstack.conf, the node that this is run on will be
    /// drained.
    fn job_prolog(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        enter!();
        if spank.context()? == Context::Remote {
            dump_context!(spank);
        }
        Ok(())
    }

    /// Called at the same point as slurm_spank_init, but after all user options
    /// to the plugin have been processed.
    ///
    /// The reason that the init and init_post_opt callbacks are separated is so
    /// that plugins can process system-wide options specified in plugstack.conf
    /// in the init callback, then process user options, and finally take some
    /// action in slurm_spank_init_post_opt if necessary. In the case of a
    /// heterogeneous job, slurm_spank_init is invoked once per job component.
    fn init_post_opt(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        // Check if the option was set
        enter!();
        if spank.context()? == Context::Remote {
            dump_context!(spank);
        }
        self.value = spank
            .get_option_value("rust-ske-option")
            .wrap_err("Failed to read --rust-ske-option option")?
            .map(|s| s.to_string());
        if let Some(value) = &self.value {
            info!("rust-ske-option = {value}");
            if spank.context()? == Context::Remote {
                spank.setenv("SPANK_RUST_SKELETON_ENVVAR", value, true)?;
            }
        }
        Ok(())
    }

    /// Called in local (srun) context only after all options have been
    /// processed.
    ///
    /// This is called after the job ID and step IDs are available. This happens
    /// in srun after the allocation is made, but before tasks are launched.
    fn local_user_init(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        enter!();
        dump_context!(spank);
        Ok(())
    }

    /// Called after privileges are temporarily dropped. (remote context only)
    fn user_init(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        enter!();
        dump_context!(spank);
        Ok(())
    }

    /// Called for each task just after fork, but before all elevated privileges
    /// are dropped. (remote context only)
    fn task_init_privileged(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        enter!();
        dump_context!(spank);
        Ok(())
    }

    /// Called for each task just before execve (2).
    ///
    /// If you are restricting memory with cgroups, memory allocated here will be
    /// in the job's cgroup. (remote context only)
    fn task_init(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        enter!();
        dump_context!(spank);
        if let Ok(result) = spank.job_env() {
            // dump job environment variables
            info!("{:#?}", result);
        }
        Ok(())
    }

    /// Called for each task from parent process after fork (2) is complete.
    ///
    ///  Due to the fact that slurmd does not exec any tasks until all tasks
    ///  have completed fork (2), this call is guaranteed to run before the user
    ///  task is executed. (remote context only)
    fn task_post_fork(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        enter!();
        dump_context!(spank);
        Ok(())
    }

    /// Called for each task as its exit status is collected by Slurm. (remote context only)
    fn task_exit(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        enter!();
        dump_context!(spank);

        if let Ok(result) = spank.task_exit_status() {
            info!("exit status = {}", result);
        }
        Ok(())
    }

    /// Called at the same time as the job epilog.
    ///
    /// If this function returns an error and the SPANK plugin that contains it
    /// is required in the plugstack.conf, the node that this is run on will be
    /// drained.
    fn job_epilog(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        enter!();
        if spank.context()? == Context::Remote {
            dump_context!(spank);
        }
        Ok(())
    }

    /// Called in slurmd when the daemon is shut down.
    fn slurmd_exit(&mut self, _spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        enter!();
        Ok(())
    }

    /// Called once just before slurmstepd exits in remote context. In local
    /// context, called before srun exits.
    fn exit(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        enter!();
        if spank.context()? == Context::Remote {
            dump_context!(spank);
            if let Ok(result) = spank.getenv("SPANK_RUST_SKELETON_ENVVAR") {
                info!(
                    "SPANK_RUST_SKELETON_ENVVAR = {}",
                    result.unwrap_or("".to_string())
                );
            }
        }
        Ok(())
    }
}
