##### - configuration - #####
NETWORK_NAME="testnet" # devnet, testnet, mainnet
PROXY=https://testnet-gateway.elrond.com
CHAIN_ID="T"

WALLET="./wallets/testnet-wallet.pem" # main actor pem file

TOKEN_NAME="LAND-2685e5"
LOCKED_TOKEN_NAME="LKLAND-bad786"

TOKEN_NAME_HEX="0x$(echo -n ${TOKEN_NAME} | xxd -p -u | tr -d '\n')"
LOCKED_TOKEN_NAME_HEX="0x$(echo -n ${LOCKED_TOKEN_NAME} | xxd -p -u | tr -d '\n')"
TOKEN_PRICE=200000000000000
MIN_BUY_LIMIT=200000000000000000
MAX_BUY_LIMIT=1000000000000000000



######
ADDRESS=$(erdpy data load --key=address-testnet)
######

deploy() {
    erdpy --verbose contract deploy \
    --project=${PROJECT} \
    --recall-nonce \
    --pem=${WALLET} \
    --gas-limit=100000000 \
    --arguments ${TOKEN_NAME_HEX} ${LOCKED_TOKEN_NAME_HEX} ${TOKEN_PRICE} ${MIN_BUY_LIMIT} ${MAX_BUY_LIMIT} \
    --send \
    --outfile="deploy-testnet.interaction.json" \
    --proxy=${PROXY} \
    --metadata-payable \
    --metadata-payable-by-sc \
    --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-testnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}


start_sale() {
    erdpy --verbose contract call ${ADDRESS} \
    --recall-nonce \
    --pem=${WALLET} \
    --gas-limit=5000000 \
    --function="startSale" \
    --send \
    --proxy=${PROXY} \
    --chain=${CHAIN_ID}
}

get_token_price() {
    erdpy --verbose contract call ${ADDRESS} \
    --recall-nonce \
    --pem=${WALLET} \
    --gas-limit=5000000 \
    --function="getTokenPrice" \
    --send \
    --proxy=${PROXY} \
    --chain=${CHAIN_ID}
}