use std::collections::BTreeSet;

pub fn parse_cpuset(cpuset_config: &str) -> BTreeSet<u32> {
    let mut cpus = Vec::new();
    for part in cpuset_config.split(',') {
        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() == 2 {
                if let (Result::Ok(start), Result::Ok(end)) =
                    (range[0].parse::<u32>(), range[1].parse::<u32>())
                {
                    cpus.extend(start..=end);
                }
            }
        } else if let Result::Ok(num) = part.parse::<u32>() {
            cpus.push(num);
        }
    }

    BTreeSet::from_iter(cpus.into_iter())
}
