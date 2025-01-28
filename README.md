# Blockclock Backend & Solver

This is a service that provides the blockclock backend and solver functionality.

## Building

1.  Make sure that `docker` and `gcloud` are installed on your local machine.
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
	    "avatar": "<time_keeper_name>",
    }
    ```
1.  `/add_time_sig`

    Thre `POST` request, to be called every time when the time leeper issues a new time signature.

    Body:

    ```json
    {
      "epoch": "<Unix epoch in nanoseconds>",
      "time_keeper": "<The time keeper address>",
      "signature": "<ECDSA signature, 65 bytes>"
    }
    ```
1.  `/claim_avatar`

    The `POST` request, should be called when the time keeper claims a new name.

    Body:

    ```json
    {
      "time_keeper": "<Address>",
      "avatar": "<New name>"
    }
    ```
1.  `/list_time_sigs`

    The `GET` request, debug output of all existing time signatures in tne memory pool.

    Params: None
1.  `/get_time_margin`

    The `GET` request, returns the current time margin used for mean time computing.
    Expected response JSON:
    ```json
    {
        "time_margin": "<margin in nanoseconds>"
    }
    ```
1.  `/get_time_keepers_count`

    The `GET` request, returns a number of time keepers contributing to the blockclock.
    Expected response JSON example:
    ```json
    {
        "count": 42
    }
    ```

1.  `/update_referral_code`

    The `POST` request, updates the account's referral code. Can be used for setting a custom referral code.

    Body:

    ```json
    {
        "time_keeper": "<Address>",
        "referral_code": "<A new referral code>"
    }
    ```

1.  `/update_referred_from`

    The `POST` request, updates the referral code that made referral for this account.

    Body:

    ```json
    {
        "time_keeper": "<Address>",
        "referred_from": "<A referral code of this account's referrer>"
    }
    ```

1.  `/write_referral`

    The `POST` request, puts a device's ID and a referral code into referrers table. Expected on the making referral process, to be called by the referral web app.

    Body:

    ```json
    {
        "refkey": "<Device ID, computed by the referral web app>",
        "refvalue": "<Referral code>"
    }
    ```

1.  `/read_referral`

    The `GET` request, requests a referral ID of the referrer who referred this user. Expected to be called by the application on the onboarding. The retrtieved referral ID is to be stored afterwards.

    Request:

    `/read_referral?ref_key=<device ID>`

    Expected response:

    ```json
    {
        "refkey": "<Device ID>",
        "refvalue": "<Referral code>"
    }
    ```