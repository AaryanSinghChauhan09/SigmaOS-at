//! At model — schedule commands for later execution.

#[derive(Debug, Clone)]
pub struct Job {
    pub id: u64,
    pub command: String,
    pub run_at: String,
    pub queue: String,
}

#[derive(Debug, Clone)]
pub struct At {
    pub queue: String,
    pub next_id: u64,
    pub jobs: Vec<Job>,
}

impl Default for At {
    fn default() -> Self { Self { queue: "a".into(), next_id: 1, jobs: vec![] } }
}

impl At {
    pub fn new() -> Self { Self::default() }

    pub fn schedule(&mut self, command: &str, run_at: &str) -> Job {
        let job = Job { id: self.next_id, command: command.to_string(), run_at: run_at.to_string(), queue: self.queue.clone() };
        self.next_id += 1;
        self.jobs.push(job.clone());
        job
    }

    pub fn list(&self) -> Vec<&Job> {
        self.jobs.iter().collect()
    }

    pub fn remove(&mut self, id: u64) -> bool {
        let len = self.jobs.len();
        self.jobs.retain(|j| j.id != id);
        self.jobs.len() != len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn schedule_list_remove() {
        let mut a = At::new();
        let j = a.schedule("backup.sh", "03:00");
        assert_eq!(a.list().len(), 1);
        assert!(a.remove(j.id));
        assert!(a.list().is_empty());
    }
}
