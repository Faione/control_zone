use clap::ValueEnum;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SCHED {
    NORMAL = 0,
    FIFO = 1,
    RR = 2,
    BATCH = 3,
    IDLE = 5,
    DEADLINE = 6,
    EXT = 7,
}

impl From<i32> for SCHED {
    fn from(value: i32) -> Self {
        match value {
            0 => SCHED::NORMAL,
            1 => SCHED::FIFO,
            2 => SCHED::RR,
            3 => SCHED::BATCH,
            5 => SCHED::IDLE,
            6 => SCHED::DEADLINE,
            7 => SCHED::EXT,
            _ => panic!("invalid policy: {}", value),
        }
    }
}

impl Into<i32> for SCHED {
    fn into(self) -> i32 {
        match self {
            SCHED::NORMAL => 0,
            SCHED::FIFO => 1,
            SCHED::RR => 2,
            SCHED::BATCH => 3,
            SCHED::IDLE => 5,
            SCHED::DEADLINE => 6,
            SCHED::EXT => 7,
        }
    }
}

#[cfg(test)]
mod test {
    use clap::ValueEnum;

    use super::SCHED;

    #[test]
    fn to_string() {
        let sched_class = SCHED::NORMAL;
        let name = sched_class
            .to_possible_value()
            .map(|pv| String::from(pv.get_name()))
            .unwrap();

        assert_eq!(name, "normal");
    }
}
