 #!/bin/bash

docker run --env-file .env --expose 8080 -p 8080:8080 heosd
