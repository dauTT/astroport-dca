#!/usr/bin/env bash

# images:
# - terra-local_astroport
# - astroport:v1.0.0
# - dautt/astroport:v1.0.0
# create image -->  docker commit terra-local_terrad_1 dautt/astroport:v1.0.0
# push to hub.docker --> docker push  dautt/astroport:v1.0.0

# cmd: 
# clean -> clean container
# stop --> stop container 
# run --> run image 
# enter --> enter into the running container
# commit --> create a new image from a container name: 
#            $2: [container name], $3: [name new image]


# set the image to run
IMAGE=terra-local_astroport


if [[ "$1" = "stop" ]]; then
    docker stop $IMAGE
fi

if [[ "$1" = "clean" ]]; then
    docker rm -f $IMAGE
fi

if [[ "$1" = "run" ]]; then
    docker run -d \
        --name $IMAGE \
        -p 1317:1317 \
        -p 26656:26656 \
        -p 26657:26657 \
        -p 9090:9090 \
        -p 9091:9091 \
        -v $PWD/config:/root/.terra/config \
        $IMAGE ;

    echo $IMAGE
fi

if [[ "$1" = "enter" ]]; then
    docker exec -it $IMAGE /bin/sh 
fi

# create a new image from an existing container name
if [[ "$1" = "commit" ]]; then
    docker stop  $2; 
    docker commit $2  $3
fi
 




