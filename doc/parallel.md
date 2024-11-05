# Parallel Communication and Setup

## Initialization

Manager | Worker
--|--
Spawn workers | --
Recv | Send ThreadID
Send all thread_ids | Recv
Send run function | Recv
-- | Create sim, connect workers, run fn