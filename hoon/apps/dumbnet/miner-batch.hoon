/=  mine  /common/pow
/=  pow-parallel  /common/pow-parallel
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
    ::  Use parallel proof generation
    =/  result  (prove-block-parallel:pow-parallel length.batch block-commitment.batch nonces.batch)
    ?~  result
      ~>  %slog.[0 [%leaf "batch miner: no valid proof found"]]
      `k
    ::  Found a valid proof
    =/  [prf=proof:sp dig=tip5-hash-atom nonce=noun-digest:tip5]  u.result
    ~>  %slog.[0 [%leaf "batch miner: found valid proof!"]]
    :_  k
    [%command %pow prf dig block-commitment.batch nonce]~
  --
-- 
