# Blockclock Backend & Solver

This is a service that provides the blockclock backend and solver functionality.

## Building

1.  Make sure that `docker` abd `gcloud` are installed on your local machine.
1.  Run `./build_image.sh`.
    1.  Choose the environment from the prompt Use prod if you're updating the production version.
    1.  It might ask for logging in to Google account. Use your stxn accounbt for login.
    1.  The docker image will be built.
1.  If you encounter error due to a lack of gcloud permissions, ping the gcloud admin, we'll grant necessary permissions.

## Deploying

1.  Modify the `./deploy.sh` script if you need to change some params, e.g. contract addresses.
1.  Login to the `google cloud console` with your stxn account.
1.  ssh into the `blockclock-solver` VM.
1.  Copy the deploy.sh into the default home directory. Replace existing script if it exists.
1.  Run the ./deploy.sh on the VM 

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

    Params: None