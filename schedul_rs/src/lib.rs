#[macro_use]
extern crate nickel;
extern crate lazy_static;

/* TODO
 *  - [ ] Cleanup code (practice rust)
 *      - [ ] Use a logger
 *      - [ ] Make deployment details run func params
 *      - [ ] Decide if functional or oo, and stick to it
 *  - [x] Update tests and docs
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
