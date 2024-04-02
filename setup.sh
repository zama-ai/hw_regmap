#!/bin/bash

# Export GIT_VERSION
# Associated env-var is used during the build phase
export GIT_VERSION=$(git rev-parse HEAD)
