from datetime import datetime
import time
from github import Github

TOKEN = open('secret.txt', 'r').read()

g = Github(TOKEN, per_page=1000, retry=1)

def query(start, end):
    format = '%Y-%m-%d'
    return f'language:python created:{datetime.utcfromtimestamp(start).strftime(format)}..{datetime.utcfromtimestamp(end).strftime(format)}'

output = open('repos.txt', 'w')
log = open('log.txt', 'w')

step = 604800 # One week
end_date = time.time()
start_date = time.time() - step

n = 0
for j in range(1000000):
    q = query(start_date, end_date)
    print(f'Query: "{q}"')
    result = g.search_repositories(q, sort='stars')
    for repo in result:
        n += 1
        print(f'{n:06}: {repo.clone_url}')
        log.write(f'{n:06}: {repo.clone_url}')
        output.write(f'{repo.clone_url}\n')
        if n >= 999999:
            break
    end_date -= step
    start_date -= step
