#[macro_use]
extern crate nickel;
extern crate lazy_static;

/* TODO
 *  - [ ] Cleanup code (practice rust)
 *      - [ ] Use a logger
 *      - [ ] Make deployment details run func params
 *      - [ ] Decide if functional or oo, and stick to it
 *  - [x] Final diagram overview
 *  - [x] (follower) on startup, schedule all non-taken jobs from db
 *  - [x] Support for different scheduling input types (timestamp, datetime,)
 *  - [x] Support for cron jobs/recurring jobs
 *  - [ ] Add 'job_label' and 'job_payload' to request and internals
 *  - [ ] Replace 'job_id' with optional 'schedule_id' - generate uuid on leader if not provided
 *  - [ ] Add ttl for taken jobs in db
 *  - [ ] Add end time for cron jobs
 *  - [ ] Follower setup loads cron AND untaken jobs from db
 *  - [ ] Benchmarking
 */

mod sutils;
pub use crate::sutils::utils;

pub mod follower;
pub mod leader;
pub mod redoer;
