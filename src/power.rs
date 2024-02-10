use std::fmt::Display;

use anyhow::Result;
use x86::msr;

use crate::winring0::{rdmsr, wrmsr};
pub trait Regfile {
    fn check(&self) -> Result<()>;
    fn to_reg(&self) -> Result<u64>;
    fn from_reg(reg: u64) -> Self;
}

#[derive(Debug, PartialEq)]
pub struct PowerUnit {
    /// Power related information (in Watts) is based on the multiplier, 1/ 2^PU; where PU is
    /// an unsigned integer represented by bits 3:0. Default value is 0011b, indicating power
    /// unit is in 1/8 Watts increment.
    pu: u8,
    /// Energy related information (in Joules) is based on the multiplier, 1/2^ESU;
    /// where ESU is an unsigned integer represented by bits 12:8. Default value is 10000b,
    /// indicating energy status unit is in 15.3 micro-Joules increment.
    esu: u8,
    /// Time related information (in Seconds) is based on the multiplier, 1/ 2^TU;
    /// where TU is an unsigned integer represented by bits 19:16. Default value is 1010b,
    /// indicating time unit is in 976 micro-seconds increment.
    tu: u8,
}

impl Display for PowerUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pu_watts = 1.0 / (1 << self.pu) as f64;
        let esu_joules = 1.0 / (1 << self.esu) as f64;
        let tu_seconds = 1.0 / (1 << self.tu) as f64;
        write!(
            f,
            "Power unit: {} W, Energy status unit: {} J, Time unit: {} s",
            pu_watts, esu_joules, tu_seconds
        )
    }
}

impl Regfile for PowerUnit {
    fn check(&self) -> Result<()> {
        // pu should be at most 4 bits
        if self.pu > 0xf {
            return Err(anyhow::anyhow!("Invalid pu"));
        }
        // esu should be at most 5 bits
        if self.esu > 0x1f {
            return Err(anyhow::anyhow!("Invalid esu"));
        }
        // tu should at most 4 bits
        if self.tu > 0xf {
            return Err(anyhow::anyhow!("Invalid tu"));
        }
        Ok(())
    }
    fn to_reg(&self) -> Result<u64> {
        self.check()?;
        Ok((self.pu as u64) | ((self.esu as u64) << 8) | ((self.tu as u64) << 16))
    }

    fn from_reg(reg: u64) -> Self {
        PowerUnit {
            pu: (reg & 0x0f) as u8,
            esu: ((reg >> 8) & 0x1f) as u8,
            tu: ((reg >> 16) & 0x0f) as u8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PowerLimit {
    /// Power limit 1, unit by the “Power Units” field of MSR_RAPL_POWER_UNIT
    pl1: u16,
    /// Whether power limit 1 is enabled.
    enable_pl1: bool,
    /// Whether power limit 1 is clamped. If true, the cpu would strictly follow the power limit 1 setting. Otherwise it may refuse to further reduce frequency if turbo has already been disabled.
    clamp_pl1: bool,
    /// Time limit = 2^Y * (1.0 + Z/4.0) * Time_Unit
    /// Here “Y” is the unsigned integer value represented. by bits 4:0, “Z” is an unsigned integer represented by
    /// bits 6:5. “Time_Unit” is specified by the “Time Units” field of MSR_RAPL_POWER_UNIT
    ///
    /// pl1 actually does not have a time limit
    /// This is for pl2
    time_pl1: u8,
    /// Power limit 2, unit by the “Power Units” field of MSR_RAPL_POWER_UNIT
    pl2: u16,
    /// Whether power limit 2 is enabled.
    enable_pl2: bool,
    /// Whether power limit 2 is clamped. If true, the cpu would strictly follow the power limit 2 setting. Otherwise it may refuse to further reduce frequency if turbo has already been disabled.
    clamp_pl2: bool,
    /// Time window for power level 2. Time limit = 2^Y * (1.0 + Z/4.0) * Time_Unit
    /// Here “Y” is the unsigned integer value represented. by bits 4:0, “Z” is an unsigned integer represented by
    /// bits 6:5. “Time_Unit” is specified by the “Time Units” field of MSR_RAPL_POWER_UNIT. This field may have
    /// a hard-coded value in hardware and ignores values written by software.
    ///
    /// This does not work at all, the time of pl2 is determines by pl1_time, lol.
    time_pl2: u8,
}

impl Regfile for PowerLimit {
    fn check(&self) -> Result<()> {
        // pl1 should be at most 15 bits
        if self.pl1 > 0x7fff {
            return Err(anyhow::anyhow!("Invalid pl1"));
        }
        // pl1 time should be at most 7 bits
        if self.time_pl1 > 0x7f {
            return Err(anyhow::anyhow!("Invalid time_pl1"));
        }
        // pl2 should be at most 15 bits
        if self.pl2 > 0x7fff {
            return Err(anyhow::anyhow!("Invalid pl2"));
        }
        // pl2 time should be at most 7 bits
        if self.time_pl2 > 0x7f {
            return Err(anyhow::anyhow!("Invalid time_pl2"));
        }
        Ok(())
    }
    fn to_reg(&self) -> Result<u64> {
        self.check()?;
        let pl1_reg = (self.pl1 as u64)
            | ((self.enable_pl1 as u64) << 15)
            | ((self.clamp_pl1 as u64) << 16)
            | ((self.time_pl1 as u64) << 17);
        let pl2_reg = (self.pl2 as u64)
            | ((self.enable_pl2 as u64) << 15)
            | ((self.clamp_pl2 as u64) << 16)
            | ((self.time_pl2 as u64) << 17);
        let reg = pl1_reg | (pl2_reg << 32);
        Ok(reg)
    }
    fn from_reg(reg: u64) -> Self {
        PowerLimit {
            pl1: (reg & 0x7fff) as u16,
            enable_pl1: (reg & (1 << 15)) != 0,
            clamp_pl1: (reg & (1 << 16)) != 0,
            time_pl1: ((reg >> 17) & 0x7f) as u8,
            pl2: ((reg >> 32) & 0x7fff) as u16,
            enable_pl2: (reg & (1 << 47)) != 0,
            clamp_pl2: (reg & (1 << 48)) != 0,
            time_pl2: ((reg >> 49) & 0x7f) as u8,
        }
    }
}

/// Wrapper for power status
/// Method on this struct will keep power_unit unmodified and only change power_limit
#[derive(Debug, PartialEq)]
pub struct PowerStatus {
    pub power_unit: PowerUnit,
    pub power_limit: PowerLimit,
}

impl Display for PowerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n", self.power_unit)?;
        let pl1_watts = self.power_limit.pl1 >> self.power_unit.pu;
        let pl1_time_y = self.power_limit.time_pl1 & 0x1f;
        let pl1_time_z = (self.power_limit.time_pl1 >> 5) & 0x3;
        let pl1_time = 2u64.pow(pl1_time_y as u32) as f64 * (1.0 + pl1_time_z as f64 / 4.0)
            / (1 << self.power_unit.tu) as f64;
        let pl2_watts = self.power_limit.pl2 >> self.power_unit.pu;
        let pl2_time_y = self.power_limit.time_pl2 & 0x1f;
        let pl2_time_z = (self.power_limit.time_pl2 >> 5) & 0x3;
        let pl2_time = 2u64.pow(pl2_time_y as u32) as f64 * (1.0 + pl2_time_z as f64 / 4.0)
            / (1 << self.power_unit.tu) as f64;
        write!(
            f,
            "Power limit 1: {} W, time: {} s, enable: {}, clamp: {}\nPower limit 2: {} W, time: {} s, enable: {}, clamp: {}",
            pl1_watts, pl1_time, self.power_limit.enable_pl1, self.power_limit.clamp_pl1,
            pl2_watts, pl2_time, self.power_limit.enable_pl2, self.power_limit.clamp_pl2)
    }
}

impl PowerStatus {
    pub fn read_from_msr() -> Result<PowerStatus> {
        let power_unit = rdmsr(msr::MSR_RAPL_POWER_UNIT)?;
        let power_limit = rdmsr(msr::MSR_PKG_POWER_LIMIT)?;
        Ok(PowerStatus {
            power_unit: PowerUnit::from_reg(power_unit),
            power_limit: PowerLimit::from_reg(power_limit),
        })
    }
    pub fn write_to_msr(&self) -> Result<()> {
        // let power_unit = self.power_unit.to_reg()?;
        let power_limit = self.power_limit.to_reg()?;
        // wrmsr(msr::MSR_RAPL_POWER_UNIT, power_unit)?;
        wrmsr(msr::MSR_PKG_POWER_LIMIT, power_limit)?;
        Ok(())
    }
    pub fn set_pl1(&mut self, watts:u16){
        self.power_limit.pl1 = watts << self.power_unit.pu;
    }
    pub fn set_pl2(&mut self, watts:u16){
        self.power_limit.pl2 = watts << self.power_unit.pu;
    }
    pub fn set_pl2_time(&mut self, time:f64){
        let time = time * (1 << self.power_unit.tu) as f64;
        let y = (time.log2() as u8) & 0x1f;
        let z = ((time / 2u16.pow(y as u32) as f64 - 1.0) * 4.0) as u8;
        self.power_limit.time_pl1 = y | (z << 5);
    }
    pub fn set_pl1_clamp(&mut self, clamp:bool){
        self.power_limit.clamp_pl1 = clamp;
    }
    pub fn set_pl2_clamp(&mut self, clamp:bool){
        self.power_limit.clamp_pl2 = clamp;
    }
}

#[cfg(test)]
mod test {
    use super::Regfile;
    #[test]
    fn should_read_from_msr() {
        let power_status = super::PowerStatus::read_from_msr().unwrap();
        println!("{}", power_status);
        let power_limit_reg = power_status.power_limit.to_reg().unwrap();
        let power_limit = super::PowerLimit::from_reg(power_limit_reg);
        assert_eq!(power_status.power_limit, power_limit);
        let power_unit_reg = power_status.power_unit.to_reg().unwrap();
        let power_unit = super::PowerUnit::from_reg(power_unit_reg);
        assert_eq!(power_status.power_unit, power_unit);
    }
}
