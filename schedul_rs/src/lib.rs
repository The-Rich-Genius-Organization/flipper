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
 *  - [ ] Support for different scheduling input types (timestamp, datetime,)
 *  - [ ] Support for cron jobs/recurring jobs
 *  - [ ] Benchmarking
 */

mod sutils;
pub use crate::sutils::utils;

pub mod follower;
pub mod leader;
