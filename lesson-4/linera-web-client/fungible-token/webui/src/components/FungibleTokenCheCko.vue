<template>
  <div>
    <p class='text-h1'>Fungible Token</p>
    <p class='text-h6'>Faucet Chain: {{ faucetChain }}</p>
    <p class='text-h6'>Default Chain: {{ defaultChain }}</p>
    <p class='text-h6'>Application: {{ _applicationId }}</p>
    <p class='text-h6'>Total Supply: {{ _totalSupply }}</p>
    <p class='text-h6'>Supply Balance: {{ _balance }}</p>
    <p class='text-h6'>From: {{ chainId }}@{{ publicKey }} {{ _chainBalance }} / {{ memeBalance }}</p>
    <p class='text-h6'>To: {{ targetChainId }}@0xB835fc25421e08E4CBe8adb3eb7D8c8B0A2D777F</p>
    <q-btn @click='onMintClick' class='text-h4'>Mint</q-btn>
    <q-btn @click='onTransferClick' class='text-h4'>Transfer</q-btn>
    <q-btn @click='onRefreshClick' class='text-h4'>Refresh</q-btn>
  </div>
</template>

<script setup lang="ts">
import axios from 'axios'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { Subscription } from 'src/subscription'
import { keccak } from 'hash-wasm'
import { gql } from '@apollo/client'
import { Web3 } from 'web3'
import * as lineraWasm from '../../dist/wasm/linera_wasm'
import initWasm from '../../dist/wasm/linera_wasm'
import wasmModuleUrl from '../../dist/wasm/linera_wasm_bg.wasm?url'
import { stringify } from 'lossless-json'

const _balance = ref('0')
const _totalSupply = ref('0')

const _applicationId = ref('')
const defaultChain = ref('')
const faucetChain = ref('8fd4233c5d03554f87d47a711cf70619727ca3d148353446cab81fb56922c9b7')
const defaultChainUrl = computed(() => `http://10.1.24.80:8080/chains/${defaultChain.value}/applications/${_applicationId.value}`)

const chainId = ref('')
const publicKey = ref('')
const _chainBalance = ref('')
const accountBalance = ref('')
const memeBalance = ref('')
const targetChainId = ref('')
const subscriptionId = ref(undefined as unknown as string)

const metadata = async (url: string) => {
  const resp = await axios.post(url, {
    'query': 'query { balance totalSupply symbol }'
  })
  const data = (resp.data as Record<string, unknown>).data
  return {
    balance: (data as Record<string, string>).balance as string,
    totalSupply: (data as Record<string, string>).totalSupply as string,
    ticker: (data as Record<string, string>).symbol as string
  }
}

const MINT = gql`
  mutation mint($amount: Amount!) {
    mint(amount: $amount)
  }
`

const onMintClick = async () => {
  const variables = {
    amount: '1.235'
  }
  const queryBytes = await lineraWasm.graphql_deserialize_fungible_token_operation(MINT.loc?.source?.body as string, stringify(variables) as string)
  window.linera.request({
    method: 'linera_graphqlMutation',
    params: {
      applicationId: _applicationId.value,
      publicKey: publicKey.value,
      query: {
        query: MINT.loc?.source?.body,
        variables,
        applicationOperationBytes: queryBytes
      },
      operationName: 'Mint'
    }
  }).then(() => {
    myBalance()
  }).catch((e) => {
    console.log(e)
  })
}

const TRANSFER = gql`
  mutation transfer($to: Account!, $amount: Amount!) {
    transfer(to: $to, amount: $amount)
  }
`

const onTransferClick = async () => {
  const variables = {
    to: targetChainId.value,
    amount: '1.235'
  }
  const queryBytes = await lineraWasm.graphql_deserialize_fungible_token_operation(TRANSFER.loc?.source?.body as string, stringify(variables) as string)
   window.linera.request({
    method: 'linera_graphqlMutation',
    params: {
      applicationId: _applicationId.value,
      publicKey: publicKey.value,
      query: {
        query: TRANSFER.loc?.source?.body,
        variables,
        applicationOperationBytes: queryBytes
      }
    }
  }).then((result) => {
    console.log(result)
  }).catch((e) => {
    console.log(e)
  })
}

const onRefreshClick = () => {
  void myBalance()
}

const getChains = async () => {
  const url = 'http://10.1.24.80:8080'
  const resp = await axios.post(url, {
    'query': 'query { chains { list default } }'
  })
  const data = (resp.data as Record<string, unknown>).data
  defaultChain.value =  ((data as Record<string, string>).chains as unknown as Record<string, string>).default as string

  const wsUrl = 'ws://10.1.24.80:8080/ws'
  new Subscription(url, wsUrl, defaultChain.value, (hash: string) => {
    console.log(`NewBlock ${hash} on chain ${defaultChain.value}`)
    metadata(defaultChainUrl.value).then((_metadata) => {
      _balance.value = _metadata.balance
    }).catch((e) => {
      console.log('Error', e)
    })
  })
}

const MY_BALANCE = gql`
  query myBalance { myBalance }
`

const myBalance = () => {
  console.log(`Query balance of ${chainId.value}`)
  // TODO: query balance from CheCko

  window.linera.request({
    method: 'linera_graphqlQuery',
    params: {
      applicationId: _applicationId.value,
      publicKey: publicKey.value,
      query: {
        query: MY_BALANCE.loc?.source?.body,
        variables: {
          chainId: chainId.value,
          publicKey: publicKey.value
        }
      },
      operationName: 'myBalance'
    }
  }).then((balance) => {
    memeBalance.value = balance as string
  }).catch((e) => {
    console.log('Failed query my balance: ', e)
  })
}

const applicationId = async () => {
  const url = 'http://10.1.24.80:8080'
  const resp = await axios.post(url, {
    'query': `query { applications(chainId: "${defaultChain.value}") { link } }`
  })
  const data = (resp.data as Record<string, unknown>).data
  const link = ((data as Record<string, string>).applications as unknown as Record<string, string>[])[0]?.link as string
  const parts = link.split('/')
  return parts[parts.length - 1] as string
}

const toBytes = (hex: string) => {
  if (hex.length % 2 !== 0) {
    throw Error('Must have an even number of hex digits to convert to bytes')
  }
  const numBytes = hex.length / 2
  const bytes = new Uint8Array(numBytes)
  for (let i = 0; i < numBytes; i++) {
    bytes[i] = parseInt(hex.substring(i * 2, (i + 1) * 2), 16)
  }
  return bytes
}

const ownerFromPublicKey = async (publicKey: string) => {
  const publicKeyBytes = toBytes(publicKey)
  const typeNameBytes = new TextEncoder().encode('Ed25519PublicKey::')
  const bytes = new Uint8Array([...typeNameBytes, ...publicKeyBytes])
  return await keccak(bytes, 256)
}

const BALANCES = gql`
  query balances($chainOwners: [ChainOwners!]!) {
    balances(chainOwners: $chainOwners)
  }
`

const formalizeOwner = (owner: string) => {
  return owner.startsWith('0x') ? owner : `0x${owner}`
}

type Balances = Record<
  string,
  {
    chainBalance: string
    ownerBalances: Record<string, string>
  }
>

const ownerBalance = (
  balances: Balances,
  chainId: string,
  owner: string
) => {
  return balances[chainId]?.ownerBalances[owner] || '0.'
}

const chainBalance = (balances: Balances, chainId: string) => {
  return balances[chainId]?.chainBalance || '0.'
}

const getBalances = async () => {
  if (!publicKey.value) return
  const owner = await ownerFromPublicKey(publicKey.value)
  window.linera.request({
    method: 'linera_graphqlQuery',
    params: {
      publicKey: publicKey.value,
      query: {
        query: BALANCES.loc?.source?.body,
        variables: {
          chainOwners: [{
            chainId: chainId.value,
            owners: [formalizeOwner(owner)]
          }],
          chainId: chainId.value,
          publicKey: publicKey.value
        }
      }
    }
  }).then((result) => {
    const balances = result as Balances
    _chainBalance.value = chainBalance(balances, chainId.value)
    accountBalance.value = ownerBalance(balances, chainId.value, formalizeOwner(owner))
  }).catch((e) => {
    console.log(e)
  })
}

const getProviderState = async () => {
  return new Promise((resolve, reject) => {
    window.linera.request({
      method: 'metamask_getProviderState'
    }).then(async (result) => {
      chainId.value = ((result as Record<string, string>).chainId)?.substring(2) as string
      publicKey.value = ((result as Record<string, string>).accounts)?.[0] as string
      await getBalances()
      resolve(undefined)
    }).catch((e) => {
      console.log('metamask_getProviderState', e)
      reject(new Error(e))
    })
  })
}

const connectWallet = async () => {
  try {
    console.log('Requesting accounts: ', window.linera)
    const web3 = new Web3(window.linera)
    const accounts = await web3.eth.requestAccounts()
    console.log('Requested accounts: ', accounts)
  } catch (e) {
    console.log('Failed connect wallet: ', e)
  }
}

const subscriptionHandler = (msg: unknown) => {
  const _msg = msg as Record<string, unknown>
  const data = _msg.data as Record<string, Record<string, Record<string, Record<string, Record<string, unknown>>>>>
  if (data?.result?.notifications?.reason?.NewBlock) {
    myBalance()
  }
}

onMounted(async () => {
  await getChains()
  _applicationId.value = await applicationId()

  await initWasm(await fetch(wasmModuleUrl))

  await connectWallet()
  await getProviderState()

  const _metadata = await metadata(defaultChainUrl.value)

  _balance.value = _metadata.balance
  _totalSupply.value = _metadata.totalSupply

  void myBalance()

  if (subscriptionId.value) return
  window.linera?.request({
    method: 'linera_subscribe'
  }).then((_subscriptionId) => {
    subscriptionId.value = _subscriptionId as string
    window.linera.on('message', subscriptionHandler)
  }).catch((e) => {
    console.log('Fail subscribe', e)
  })
})

onUnmounted(() => {
  if (!subscriptionId.value) return
  void window.linera?.request({
    method: 'linera_unsubscribe',
    params: [subscriptionId.value]
  })
  subscriptionId.value = undefined as unknown as string
})


</script>
