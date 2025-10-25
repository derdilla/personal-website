+++
title = "Earth Wallpaper"
description = "Live satelite imagery makes for a pretty cool wallpaper - here is how to do it."
date = 2025-10-20
template = "blog-entry.html"
+++

I just wanted to share this cool bash script I hacked together after finding an [esa API](https://view.eumetsat.int/geoserver/ows?service=WMS&request=GetMap&version=1.3.0&layers=mtg_fd:rgb_geocolour&styles=&format=image/png&crs=EPSG:4326&bbox=30,-15,59,38&width=2560&height=1440) that provides fresh satelite images every 10 minutes.

```bash
#!/usr/bin/bash

SWAYBG_PID=""

while true
do
  if [ -n "$SWAYBG_PID" ]; then
    sleep 600
  fi

  curl --output "$HOME/Desktop/.bg.png" "https://view.eumetsat.int/geoserver/ows?service=WMS\
&request=GetMap&version=1.3.0&layers=mtg_fd:rgb_geocolour\
&styles=&format=image/png&crs=EPSG:4326\
&bbox=30,-15,59,38\
&width=2560&height=1440"

  if [ -n "$SWAYBG_PID" ]; then
    kill "$SWAYBG_PID" 2>/dev/null
  fi

  swaybg -i $HOME/Desktop/.bg.png &
  SWAYBG_PID=$!
done
```

There might be a better way to do this, but putting this into my autostart config works good enough for me. It should work on most wayland based compositors, although I only tested it on the new COSMIC compositor.

You can modify the displayed part of the earth by changing the `bbox` argument (minLat, minLong, maxLat, malLon) and adjust for different monitor sizes with the `width` and `height` arguments. Just make sure you stick with correct proportions – the api doesn't do that for you.
