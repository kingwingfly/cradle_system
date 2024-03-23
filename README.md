# cradle_system

Inspired by the science fiction novel `The Three Body Program`.

`cradle_system` in that novel is used to threaten others: The cradle system will detonate a bomb if no signal is received so as babies will cry if stopping rocking the cradle.

This is a little like watchdog timer in embedded system: if the system is not working properly, the watchdog timer will reset the system. To avoid the reset, the system should periodically feed the watchdog timer.

However, the cradle system is more complex than watchdog timer: it may receive both local and remote signals.

`cradle_system` will provide `LocalCradle` and `RemoteCradle` to simulate the cradle system. And `RemoteCradle` will work through a p2p network by `libp2p`(WIP).
