use anyhow::Result;
use patina::power::PowerStatus;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about, author="AsterNighT", name = "patina")]
struct Args{
    /// Power limit 1
    #[arg(long="p1")]
    pl1: Option<u16>,
    /// Clamp power limit 1
    #[arg(long="c1")]
    pl1_clamp: Option<bool>,
    /// Power limit 2
    #[arg(long="p2")]
    pl2: Option<u16>,
    /// Clamp power limit 2
    #[arg(long="c2")]
    pl2_clamp: Option<bool>,
    /// Time for power limit 2
    #[arg(long="t")]
    time: Option<f64>,
    /// Read the current power status only
    #[arg(long="read")]
    read: bool,
    /// Dry run, do not write to msr
    #[arg(long="dry")]
    dry_run: bool,
}

fn main() -> Result<()>{
    let args = Args::parse();
    #[cfg(debug_assertions)]
    println!("{:?}", args);
    let mut power_status = PowerStatus::read_from_msr()?;
    if args.read {
        println!("{}", power_status);
        return Ok(());
    }
    modify_power_limits(&args, &mut power_status)?;
    println!("{}", power_status);
    if !args.dry_run {
        power_status.write_to_msr()?;
    }
    Ok(())
}

fn modify_power_limits(args:&Args, power_status:&mut PowerStatus) -> Result<()>{
    if let Some(pl1) = args.pl1 {
        power_status.set_pl1(pl1);
    }
    if let Some(pl2) = args.pl2 {
        power_status.set_pl2(pl2);
    }
    if let Some(pl1_clamp) = args.pl1_clamp {
        power_status.set_pl1_clamp(pl1_clamp);
    }
    if let Some(pl2_clamp) = args.pl2_clamp {
        power_status.set_pl2_clamp(pl2_clamp);
    }
    if let Some(time) = args.time {
        power_status.set_pl2_time(time);
    }
    Ok(())
}
