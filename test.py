import requests

job = {
    "job_id": "009",
    "seconds": 2
}

r = requests.post("http://localhost:7000/api/v1/job", json=job)
