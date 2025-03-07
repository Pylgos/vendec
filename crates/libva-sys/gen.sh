#!/usr/bin/env bash

cflags="$(pkg-config --cflags libva)"
bindgen \
  --dynamic-loading va \
  --allowlist-file '.*/va/.*' \
  --no-prepend-enum-name \
  --with-derive-default \
  wrapper.h \
  -- \
  $(pkg-config --cflags libva) \
  > src/bindings.rs
  
