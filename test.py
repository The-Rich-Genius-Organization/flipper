import requests
from uuid import uuid4
from datetime import datetime, timedelta

seconds = 5
timestamp = datetime.now() + timedelta(seconds=seconds)
utc = timestamp.astimezone().isoformat()
cron = "*/10 * * * * *"


jobs = [
    {
        "job_id": str(uuid4())[:8],
        "schedule_type": "seconds",
        "schedule_value": f'{seconds}',
    },
    {
        "job_id": str(uuid4())[:8],
        "schedule_type": "timestamp",
        "schedule_value": f'{timestamp.timestamp()}',
    },
    {
        "job_id": str(uuid4())[:8],
        "schedule_type": "datetime",
        "schedule_value": f'{utc}',
    },
    {
        "job_id": str(uuid4())[:8],
        "schedule_type": "cron",
        "schedule_value": cron
    }
]

for i, body in enumerate(jobs):
    body['job_id'] = f'{i}-{body["job_id"]}'
    print(body)
    r = requests.post("http://localhost:7000/api/v1/job", json=body)
    if r.status_code != 200:
        print(r.status_code, r.text)
        break
