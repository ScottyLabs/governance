import requests
import os
from dotenv import load_dotenv

load_dotenv()

token = os.getenv("AUTHENTIK_API_TOKEN")
url = os.getenv("AUTHENTIK_API_URL")


def delete_authentik_group(group_name):
    get_group_url = f"{url}/api/v3/core/groups/?name={group_name}"
    response = requests.get(
        get_group_url,
        headers={"Authorization": f"Bearer {token}"},
    )
    group_uuid = (
        response.json()["results"][0]["pk"] if response.json()["results"] else None
    )
    if group_uuid:
        delete_group_url = f"{url}/api/v3/core/groups/{group_uuid}/"
        requests.delete(
            delete_group_url,
            headers={"Authorization": f"Bearer {token}"},
        )


def create_authentik_group(group_name):
    # delete the group if it exists since Governance repo is the source of truth
    delete_authentik_group(group_name)

    create_group_url = f"{url}/api/v3/core/groups/"
    response = requests.post(
        create_group_url,
        json={"name": group_name},
        headers={"Authorization": f"Bearer {token}"},
    )
    return response.json()


def get_user(user_andrew_id):
    get_user_url = f"{url}/api/v3/core/users/?email={user_andrew_id}@andrew.cmu.edu"
    response = requests.get(
        get_user_url,
        headers={"Authorization": f"Bearer {token}"},
    )
    return response.json()


def add_user_to_group(group_uuid, user_pk):
    add_user_to_group_url = f"{url}/api/v3/core/groups/{group_uuid}/add_user/"
    requests.post(
        add_user_to_group_url,
        json={"pk": user_pk},
        headers={"Authorization": f"Bearer {token}"},
    )


group_response = create_authentik_group("TEST")
group_uuid = group_response["pk"]
# print(group_uuid)

# user_response = get_user("yh4")
# user_uuid = user_response["results"][0]["pk"]

# add_user_to_group(group_uuid, user_uuid)
