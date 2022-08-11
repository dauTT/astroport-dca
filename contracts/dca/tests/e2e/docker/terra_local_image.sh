#!/usr/bin/env bash

# images:
# - terra-local_astroport
# - astroport:v1.0.0 -- core astroport
# - dautt/astroport:v1.0.3  --> core astroport, test tokens, test pools, dca ontract
# - dautt/astroport:v1.2.0  --> core astroport, test tokens, test pools. provide liquidity, dca contract
# - dautt/astroport:v1.1.0  --> core astroport, test tokens, test pools. provide liquidity
# - dautt/astroport:v1.0.2  --> core astroport, test tokens, test pools
# - dautt/astroport:v1.0.1  --> core astroport, test tokens
# - dautt/astroport:v1.0.0
# create image -->  docker commit terra-local_terrad_1 dautt/astroport:v1.0.0
# push to hub.docker --> docker push  dautt/astroport:v1.0.0


if [[ "$1" = "help" ]]; then
echo "********* CMD: **************" 
echo "rm     -> rimove container"
echo "stop   -> stop container" 
echo "run    -> run image" 
echo "enter  -> enter into the running container"
echo "commit -> create a new image from a container name: " 
echo "          params: [name new image]"
echo "push   -> push image to repository"
fi




# set the image to run
VERSION=v1.2.0
IMAGE=dautt/astroport:$VERSION
CONTAINER_NAME=astroport_$VERSION


if [[ "$1" = "start" ]]; then
    docker start $CONTAINER_NAME
fi


if [[ "$1" = "stop" ]]; then
    docker stop $CONTAINER_NAME
fi

if [[ "$1" = "rm" ]]; then
    docker rm -f $CONTAINER_NAME
fi

if [[ "$1" = "run" ]]; then
    docker run -d \
        --name $CONTAINER_NAME \
        -p 1317:1317 \
        -p 26656:26656 \
        -p 26657:26657 \
        -p 9090:9090 \
        -p 9091:9091 \
        -v $PWD/config:/root/.terra/config \
        $IMAGE ;

    echo $CONTAINER_NAME
fi

if [[ "$1" = "enter" ]]; then
    docker exec -it $CONTAINER_NAME /bin/sh 
fi

# create a new image from an existing container name
if [[ "$1" = "commit" ]]; then
    docker stop  $CONTAINER_NAME; 
    docker commit $CONTAINER_NAME  $2
fi

# push image to repository
if [[ "$1" = "push" ]]; then
   echo "docker push "$IMAGE
   docker push $IMAGE

fi
 




