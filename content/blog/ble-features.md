+++
title = "On BLE GATT"
description = "Why I struggle to fix my app for your peripheral and your favorite features are unfeasible"
date = 2026-06-21
template = "blog-entry.html"
+++

There is a standard for the Bluetooth Low Energy Generic Attribute Profile (BLE GATT), that allows reading blood pressure measurements directly from devices that support this. I implemented this simple protocol a little over 2 years ago for my [blood pressure app](https://github.com/derdilla/blood-pressure-monitor-fl/), basically a diary for blood pressure values. I'm not sure if I've been happy with that decision...


## The Problem

The GATT is a protocol allows devices to expose services and 'characteristics' by an ID. For blood pressure that's service 1810 and characteristic 2A35. I can send a read request and the device sends me the latest measurement, and I can send an "I'm listening" flag and the device sends the measurement once it's ready.

The specification leaves it somewhat open which method is preferred, but the device must advertise which is available. For a few devices this just works: I scan, I connect, I `read`. For some others I `setIndication`, and the device calls my callback once, and I'm happy. But it is rarely that simple:


### Most timestamps are wrong

Because blood pressure measurement devices use cheap microcontrollers, clock skew is a real problem. Even modern phones
struggle with it. And for some reason nobody even attempts using the frequencies that kept radio clocks synced for decades. So it's fair to say:
whatever time the app gets from the device: it's wrong.

There are some other protocols to update the device time via BLE, but more often than not devices don't even advertise this,
or if they do, they forget the time has ever been set by the next restart.

Ok fine, we can add a setting to ignore the time they transmit and use the phone time instead.

### Some devices send multiple measurements

You may notice that in the `setIndication` interface, that callback could be called more than once. Some devices do
that: perhaps because their official apps wanted to always sync all measurements, perhaps the hardware programmer thought they had to. This is inherently doomed to fail since the timestamps are so inaccurate.

For the app that's also a problem because the BLE input is in the add measurement dialog. We only really want 1 measurement. So which do we use?
The first one? The last one? No, the order in which they are transmitted is even less specified than whether multiple measurements are allowed at all.

Ok fine, just ask the user to select the measurement they want.


### Filthy liars

Not all devices support the official protocol, some still say they do. This is really worse than a proprietary protocol.
We can reverse engineer a proprietary protocol. We can't even tell you are messing with the default implementation.

Generally these device cause stale issues where bugs are reported, I investigate, don't find anything and everyone involved feels tricked out of the time they spend.

If you design a device and really think you can't make do with the features of the official protocol. 
- Firstly: reconsider. These protocols were designed by people who probably know more about BLE, the GATT, and medicine than you do.
-  Secondly: For heaven’s sake use a different UUID, those are reserved for a reason...


### Everything outside BLE

Since the protocol only specifies how data is _transmitted_, the device can behave whatever. It might only start
Bluetooth advertising after the measurement is done, like I'd honestly expect. It might advertise other services during
measurement than after measurement. It might shut down if you try to establish the connection too early or too late.
Requesting the measurement at a bad time, might even shut the device down.


## What this leads to

All in all this just means some devices won't work. Maybe they don't deserve to, but "compatible with niche BLE app"
isn't necessarily what customers look for when they buy a monitor. In my mind, just pressing those seven number buttons in my app manually 
is no big deal, but some people disagree. With every manual selection step to fix a niche corner case that margin gets
thinner, and all that's left is the confidence that you didn't enter the wrong number, and that the app or your device
is at fault.

I get quite a few complaints about things not working where I can't do anything about it.

I still try to make people that ask for support or features (nicely) happy. Most people opening feature requests and bugs are
incredibly thankful and I'm also grateful they are interested in my app. But seeing a bug that won't get solved,
knowing that more people will lose time trying to make something work, just to find a 2 year old issue, saddens me.

Maybe I should remove that expectation from the main app? Not as long as there are still people who are happy with what works! But one day I would like to see those problems gone.

## Solutions

So why rant here? It's not like the manufacturers will do anything about it, or everyone should get a new device all of a sudden. I'm honestly not sure, maybe I just want to order my thoughts, maybe a lawmaker stumbles upon this, maybe I can [nerd snipe](https://xkcd.com/356/) others into helping me. This is really one of those cases where open source really makes sense: If each supported device 
had just one person keep their measurements running, this would be a huge win.

The other solution is, like always, money: I could just take a few thousand euros and buy every device there is. If I have physical access to a device, I am far more likely to be able to fix it.
To really get those big purpose-bound funds I'd probably need to set up a company or non-profit, which I see no point in right now. Given how niche my app is I doubt I will get to do this.

However, if you are a blood pressure device manufacturer and want my app to just work with your device:
Reach out and send me a sample, I'll see what I can do.
