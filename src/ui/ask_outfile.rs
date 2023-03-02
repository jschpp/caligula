use std::fmt;

use inquire::{Confirm, InquireError, Select};

use crate::{
    cli::Args,
    device::{BurnTarget, Removable},
};

pub fn ask_outfile(args: &Args) -> anyhow::Result<BurnTarget> {
    let mut show_all_disks = args.show_all_disks;

    loop {
        let targets = enumerate_options(show_all_disks)?;

        let ans = Select::new("Select target disk", targets)
            .with_help_message(if show_all_disks {
                "Showing all disks. Proceed with caution!"
            } else {
                "Only displaying removable disks."
            })
            .prompt()?;

        let dev = match ans {
            ListOption::Device(dev) => dev,
            ListOption::RetryWithShowAll(sa) => {
                show_all_disks = sa;
                continue;
            }
            ListOption::Refresh => {
                continue;
            }
        };

        if !confirm_write(args, &dev)? {
            continue;
        }

        return Ok(dev);
    }
}

pub fn confirm_write(args: &Args, device: &BurnTarget) -> Result<bool, InquireError> {
    if args.force {
        Ok(true)
    } else {
        println!("Input: {}", args.input.to_string_lossy());
        println!();

        println!("Output: {}", device.devnode.to_string_lossy());
        println!("  Model: {}", device.model);
        println!("  Size: {}", device.size);
        println!("  Type: {}", device.target_type);
        println!("  Removable: {}", device.removable);
        println!();

        Confirm::new("Is this okay?")
            .with_help_message("THIS ACTION WILL DESTROY ALL DATA ON THIS DEVICE!!!")
            .with_default(false)
            .prompt()
    }
}

enum ListOption {
    Device(BurnTarget),
    Refresh,
    RetryWithShowAll(bool),
}

impl fmt::Display for ListOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ListOption::Device(dev) => {
                let devnode = dev.devnode.to_string_lossy();

                write!(
                    f,
                    "{devnode} | {} - {} (removable: {})",
                    dev.model, dev.size, dev.removable
                )?;
            }
            ListOption::RetryWithShowAll(true) => {
                write!(f, "<Show all disks, removable or not>")?;
            }
            ListOption::RetryWithShowAll(false) => {
                write!(f, "<Only show removable disks>")?;
            }
            ListOption::Refresh => {
                write!(f, "<Refresh devices>")?;
            }
        }
        Ok(())
    }
}

fn enumerate_options(show_all_disks: bool) -> anyhow::Result<Vec<ListOption>> {
    let mut enumerator = udev::Enumerator::new()?;
    let devices = enumerator.scan_devices()?;

    let burn_targets = devices
        .filter_map(|d| BurnTarget::try_from(d).ok())
        .filter(|d| show_all_disks || d.removable == Removable::Yes)
        .map(ListOption::Device);

    let options = burn_targets.chain([
        ListOption::Refresh,
        ListOption::RetryWithShowAll(!show_all_disks),
    ]);

    Ok(options.collect())
}
