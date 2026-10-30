# Reports

All report functions are pure (`studyfrog-report`); `service::reports` reads the history and calls them.

## Retention
Reviews that followed an earlier review of the same item by at least a day are grouped by the gap: 1, 2, 3-4,
5-7, 8-14, 15-30, 31-90 and 91+ days. A review counts as recalled unless it was rated Again. Same-day
repeats are left out.

## Time to answer
Per item shown, the time from showing it to revealing it (or submitting an answer), paused time excluded,
read from the run event logs (practice runs included). Shown as median, 90th and 95th percentile (nearest
rank), a histogram (under 1 s, 1-2, 2-3, 3-5, 5-8, 8-13, 13-21, 21-34, 34 s or more), and the same by the
difficulty mark the user gave the item.

## Reviews ahead (forecast)
Expected reviews per day for the next 30 days. Every scheduled item is projected forward: reviewed on its
due day (overdue items count today), then after an interval that grows by the user's typical factor; every
review adds a lapse review the next day with the user's lapse rate; new items keep arriving at the pace of the
last 14 days and follow the same path from their typical first gap. The growth factor (median ratio of
successive gaps after a recalled review), the first gap, the lapse rate and the new-item pace are estimated
from the history. It is an expectation, not a promise.

## Backtest
Forecast the 28 days after a cut-off from the history up to the cut-off (replaying FSRS to find each item's
state), then compare with the reviews that really happened. Reported: the mean error of the weekly totals as a
share of the actual totals, and the bias (positive means the forecast was too high). With at least 56 days of
real history the cut-off is 28 days ago and the check uses the user's own reviews; otherwise a simulated
learner (seeded, 12 new items a day, 10 % of due reviews put off a day, FSRS recall probabilities) is used and
the report says so.

Measured on simulated histories (3 seeds, 150 days, cut at day 119): weekly totals off by 9 to 11 %, forecast
8 to 11 % low (the simulated learner puts reviews off, the forecast assumes they happen on the due day).

## Budget
Each report computes in under 50 ms on 100,000 reviews; measured 0.4 to 6.6 ms
([BENCHMARKING](../contributing/BENCHMARKING.md)).
