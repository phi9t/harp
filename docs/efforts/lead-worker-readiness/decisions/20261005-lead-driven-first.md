# Decision: the lead-driven instantiation runs first, with supervision; the controllers automate later

Date: 2026-10-05.
Authority: the owner, in the lead session, while spec draft 2 was being written:
"each instantiation have different complexity, but I'd hope we can get 1 up and running first and to pass a
good amount of tests (and we will take notes on what we need to do to get it running well; it's gonna be an
agent heavy process with some amount of babysitting and caution by the supervising agents); the latter two
will grow in sophistication, but they will allow the system to automate most work without an activate agent
to fix things".

## Accepted direction

1. The lead-driven instantiation (I1) is built and run first.
2. I1 is expected to pass a good share of the scenarios, not all of them.
3. I1 is agent-heavy by nature: supervising agents watch, are cautious and step in. An intervention in I1
   is expected and recorded; it is not by itself a failure.
4. Notes are kept on what it takes to get I1 running well. They are an output of the effort.
5. The two controllers (I2, I3) grow in sophistication afterwards. Their purpose is to automate most of the
   work without an active agent fixing things; the interventions I1 needed are what they should remove.

## Lead's reading, to be confirmed

- This is the exemption for I1 work from the controller spec's instruction to park unrelated work. The
  owner did not mention that instruction.
- Which scenarios make up "a good amount" is not named. The spec proposes a core set.
- An intervention by a supervising agent is recorded per scenario with what it did. A scenario that passed
  only because an agent stepped in is reported as "passed with intervention", distinct from "passed".
