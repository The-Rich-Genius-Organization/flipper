import requests
from uuid import uuid4

job = {
    "job_id": str(uuid4())[:8],
    "seconds": 2
}

r = requests.post("http://localhost:7000/api/v1/job", json=job)
