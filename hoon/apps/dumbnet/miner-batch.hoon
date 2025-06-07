/=  mine  /common/pow
/=  sp  /common/stark/prover
/=  *  /common/zoon
/=  *  /common/zeke
/=  *  /common/wrapper
=<  ((moat |) inner)
=>
  |%
  +$  effect  
    $%  [%command %pow prf=proof:sp dig=tip5-hash-atom block-commitment=noun-digest:tip5 nonce=noun-digest:tip5]
        [%progress done=@ total=@]  :: Progress updates
    ==
  +$  kernel-state  [%state version=%1]
  +$  batch-cause  [length=@ block-commitment=noun-digest:tip5 nonces=(list noun-digest:tip5)]
  --
|%
++  moat  (keep kernel-state)
++  inner
  |_  k=kernel-state
  ++  load
    |=  =kernel-state  kernel-state
  ++  peek
    |=  arg=*
    =/  pax  ((soft path) arg)
    ?~  pax  ~|(not-a-path+arg !!)
    ~|(invalid-peek+pax !!)
  ++  poke
    |=  [wir=wire eny=@ our=@ux now=@da dat=*]
    ^-  [(list effect) k=kernel-state]
    =/  batch  ((soft batch-cause) dat)
    ?~  batch
      ~>  %slog.[0 [%leaf "error: bad batch cause"]]
      `k
    =/  batch  u.batch
    ~>  %slog.[0 [%leaf "batch miner: processing {<(lent nonces.batch)>} nonces"]]
    ::  Process nonces to find valid proof
    ::  TODO: This needs to be parallelized via jets
    =/  results=(list [nonce=noun-digest:tip5 prf=proof:sp dig=tip5-hash-atom])
      %+  turn  nonces.batch
      |=  nonce=noun-digest:tip5
      =/  [prf=proof:sp dig=tip5-hash-atom]
        (prove-block-inner:mine length.batch block-commitment.batch nonce)
      [nonce prf dig]
    ::  For now, just return the first result
    ::  In the future, this will check targets and return first valid
    ?~  results
      ~>  %slog.[0 [%leaf "batch miner: no results"]]
      `k
    =/  [nonce=noun-digest:tip5 prf=proof:sp dig=tip5-hash-atom]  i.results
    :_  k
    [%command %pow prf dig block-commitment.batch nonce]~
  --
-- 
