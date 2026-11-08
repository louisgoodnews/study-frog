"""StudyFrog's Python API (ADR 0013): a client of the control protocol, standard library only.

    import studyfrog
    with studyfrog.connect() as sf:
        for item in sf.query("type = card and status = new"):
            print(item.title)

``connect`` attaches to a running StudyFrog (its ``run/control.sock``, or the socket a script run by
StudyFrog is given) or starts a headless one. See ``python/README.md`` and the help page "Scripting".
"""

from .client import Client, connect
from .script import access, job_name, trigger_event
from .types import Event, Item, LogEntry, StudyFrogError

__all__ = [
    "Client",
    "Event",
    "Item",
    "LogEntry",
    "StudyFrogError",
    "access",
    "connect",
    "job_name",
    "trigger_event",
]

__version__ = "0.5.0"
PROTOCOL = 1
