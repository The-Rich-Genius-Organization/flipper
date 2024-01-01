import requests
from uuid import uuid4
from datetime import datetime, timedelta
from concurrent.futures import ThreadPoolExecutor

seconds = 5
timestamp = datetime.now() + timedelta(seconds=seconds)
utc = timestamp.astimezone().isoformat()
cron = "*/10 * * * * *"

url = "http://localhost:5000/api/v1/job"


def test_single_each_type():
    jobs = [
        {
            "schedule_type": "seconds",
            "schedule_value": f'{seconds}',
            "job_label": "email",
            "job_payload": str({"from": "demo@flipper.io", "to": "jomama@yahoo.com", "subject": "Hello", "body": "Hello World"}),
        },
        {
            "schedule_type": "timestamp",
            "schedule_value": f'{timestamp.timestamp()}',
            "job_label": "payment",
            "job_payload": str({"user_id": "123", "amount": "1000", "currency": "USD"}),
        },
        {
            "schedule_type": "datetime",
            "schedule_value": f'{utc}',
            "job_label": "sms",
            "job_payload": str({"from": "1234567890", "to": "1234567890", "message": "Hello World"}),
        },
        {
            "schedule_type": "cron",
            "schedule_value": cron,
            "job_label": "etl",
            "job_payload": str({"table": "users", "columns": ["id", "name", "email"]}),
        }
    ]

    # print(jobs)
    for body in jobs:
        print(body)
        r = requests.post(url, json=body)
        print(r.text)
        assert r.status_code == 200


def _wrap_test_single_each_type(i):
    test_single_each_type()


def _test_stress():
    start = datetime.now()
    n = 100
    with ThreadPoolExecutor(max_workers=16) as executor:
        executor.map(_wrap_test_single_each_type, range(n))
    print(f"Time taken: {datetime.now() - start}")
    print(f'Avg time per job: {(datetime.now() - start) / (n * 4)}')


# test_single_each_type(0)
# _test_stress()
