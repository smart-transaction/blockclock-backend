# Blockclock Backend & Solver

This is a service that provides the blockclock backend and solver functionality.

## API Description

1.  `/onboard`
    
    The `POST` request, to be called by the app to add a new time keeper.

    Body:

    ```json
    {
	    "time_keeper": "<time_keeper_address>",
	    "avatar": "<time_keeper_name>"
    }
    ```
2.  `/add_time_sig`

    Thre `POST` request, to be called every time when the time leeper issues a new time signature.

    Body:

    ```json
    {
      "epoch": "<Unix epoch in nanoseconds>",
      "time_keeper": "<The time keeper address>",
      "signature": "<ECDSA signature, 65 bytes>"
    }
    ```
3.  `/claim_avatar`

    The `POST` request, should be called when the time keeper claims a new name.

    Body:

    ```json
    {
      "time_keeper": "<Address>",
      "avatar": "<New name>"
    }
    ```
4.  `/list_time_sigs`

    The `GET` request, debug output of all existing time signatures in tne memory pool.
