<p align="center">
    <img width="400"src="docs/flipper-logo-medium.png" alt="Flipper .rs">
</p>

A high-performance scheduling engine built in Rust. Designed for scalability and efficiency, Flipper is ideal for managing complex scheduling tasks in distributed systems.

## Features 🚀

We use a replicated leader-follower approach for horizontal scale and guaranteed execution with seamless failover.

**Schedule a job using seconds, a Unix timestamp, UTC datetime or cron expression**

> ```json
> {
>  "schedule_type": "seconds",
>  "schedule_value": "5",
>  "job_label": "email",
>  "job_payload": "{'from': 'demo@flipper.io', 'to': 'jomama@yahoo.com', 'subject': 'Hello', 'body': 'Hello World'}"
> },
> {
>  "schedule_type": "timestamp",
>  "schedule_value": "1704108914.872404",
>  "job_label": "payment",
>  "job_payload": "{'user_id': '123', 'amount': '1000', 'currency': 'USD'}"
> },
> {
>  "schedule_type": "datetime",
>  "schedule_value": "2024-01-01T18:35:14.872404+07:00",
>  "job_label": "sms",
>  "job_payload": "{'from': '1234567890', 'to': '1234567890', 'message': 'Hello World'}"
> },
> {
>  "schedule_type": "cron",
>  "schedule_value": "*/10 * * * * *",
>  "job_label": "etl",
>  "job_payload": "{'table': 'users', 'columns': ['id', 'name', 'email']}"
> }
> ```

- **Distributed Architecture**: Optimized for handling scheduling in a distributed environment.
- **High Performance**: Leveraging Rust's performance for handling concurrent tasks efficiently and reliably.
- **Scalable Design**: Containerised services easily scale with your scheduling growing demands.
- **Customizable**: Designed to be drop-in and flexible.

<p align="center">
    <img width=""src="docs/flipper-overview.png" alt="Flipper .rs">
</p>

## Dependencies :package:

- [Rust](https://www.rust-lang.org/) >= 1.7
- Docker
  - [Docker](https://www.docker.com/) ~ 24.0
  - [Docker Compose](https://docs.docker.com/compose/) ~ 2.23
- Python
  - [Python](https://www.python.org/) > 3.7
  - [Pytest](https://docs.pytest.org/en/stable/) ~ 6.2.4

## Install & Run 🛠️

```bash
docker-compose up
```

## Testing :microscope:

```bash
pytest
```

<!-- ## ️️ Common Issues and FAQ :pushpin: -->

<!-- <details> -->
<!--     <summary>Toggle Switch</summary> -->
<!--     Foldable Content[enter image description here][1] -->
<!-- </details> -->

## ️️ Upcoming Features :construction:

- [x] Job Type request field (auto created kafka topic) and Payload
- [ ] TTL for taken jobs in database

<h2 align="center">Contact</h2>
<p align='center'>
    <a href="mailto:yohanderose@gmail.com?subject=Hello">📧 Email</a>
    <a href="#">👨🏾 Website</a>
    <a href="https://www.buymeacoffee.com/yderose">🍻 Donate</a>
</p>
