<template>
  <div>
    <p class='text-h1'>Fungible Token</p>
    <p class='text-h6'>Faucet Chain: {{ faucetChain }}</p>
    <p class='text-h6'>Default Chain: {{ defaultChain }}</p>
    <p class='text-h6'>Application: {{ _applicationId }}</p>
    <p class='text-h6'>Total Supply: {{ _totalSupply }}</p>
    <p class='text-h6'>Supply Balance: {{ _balance }}</p>
    <p class='text-h6'>From: {{ chainId }}@{{ ownerFromPublicKey(publicKey) }} {{ _chainBalance }}</p>
    <p class='text-h6'>To: {{ targetChainId }}@0xB835fc25421e08E4CBe8adb3eb7D8c8B0A2D777F</p>
    <q-btn @click='onMintClick' class='text-h4'>Mint</q-btn>
    <q-btn @click='onTransferClick' class='text-h4'>Transfer</q-btn>
    <q-btn @click='onRefreshClick' class='text-h4'>Refresh</q-btn>
  </div>
</template>

<script setup lang="ts">
import axios from 'axios'
import { computed, onMounted, ref } from 'vue'
import { Subscription } from 'src/subscription'
import * as linera from '@linera/client'
import * as metamask from '@linera/metamask'
import { keccak } from 'hash-wasm'
import { gql } from '@apollo/client'

const _balance = ref('0')
const _totalSupply = ref('0')

const _applicationId = ref('')
const defaultChain = ref('')
const faucetChain = ref('8fd4233c5d03554f87d47a711cf70619727ca3d148353446cab81fb56922c9b7')
const defaultChainUrl = computed(() => `http://192.168.31.182:8080/chains/${defaultChain.value}/applications/${_applicationId.value}`)

const chainId = ref('')
const publicKey = ref('')
const _chainBalance = ref('')
const accountBalance = ref('')
const targetChainId = ref('')

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

const onMintClick = async () => {
  const query = {
    query: `
      mutation {
        mint(amount: Amount!)
      }
    `
  }

  // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access
  window.linera.request({
    method: 'linera_graphqlMutation',
    params: {
      publicKey: publicKey.value,
      query: {
        query,
        variables: {
          applicationId: _applicationId.value,
          chainId: chainId.value,
          publicKey: publicKey.value
        }
      }
    }
  }).then((result) => {
    console.log(result)
  }).catch((e) => {
    console.log(e)
  })
}

const onTransferClick = async () => {
  let query = {
    query: `
      mutation {
        transfer(to: "${targetChainId.value}", amount: "1.235")
      }
    `
  }

  // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access
  window.linera.request({
    method: 'linera_graphqlMutation',
    params: {
      publicKey: publicKey.value,
      query: {
        query,
        variables: {
          applicationId: _applicationId.value,
          chainId: chainId.value,
          publicKey: publicKey.value
        }
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
  const url = 'http://192.168.31.182:8080'
  const resp = await axios.post(url, {
    'query': 'query { chains { list default } }'
  })
  const data = (resp.data as Record<string, unknown>).data
  defaultChain.value =  ((data as Record<string, string>).chains as unknown as Record<string, string>).default as string

  const wsUrl = 'ws://192.168.31.182:8080/ws'
  new Subscription(url, wsUrl, defaultChain.value, (hash: string) => {
    console.log(`NewBlock ${hash} on chain ${defaultChain.value}`)
    metadata(defaultChainUrl.value).then((_metadata) => {
      _balance.value = _metadata.balance
    }).catch((e) => {
      console.log('Error', e)
    })
  })
}

const myBalance = () => {
  console.log(`Query balance of ${chainId.value}`)
  // TODO: query balance from CheCko

  // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access
  window.linera.request({
    method: 'linera_graphqlQuery',
    params: {
      publicKey: publicKey.value,
      query: {
        query: BALANCES.loc?.source?.body,
        variables: {
          applicationId: _applicationId.value,
          chainId: chainId.value,
          publicKey: publicKey.value
        }
      }
    }
  }).then((result) => {
    console.log(result)
  }).catch((e) => {
    console.log(e)
  })
}

const applicationId = async () => {
  const url = 'http://192.168.31.182:8080'
  const resp = await axios.post(url, {
    'query': `query { applications(chainId: "${defaultChain.value}") { link } }`
  })
  const data = (resp.data as Record<string, unknown>).data
  const link = ((data as Record<string, string>).applications as unknown as Record<string, string>[])[0]?.link as string
  const parts = link.split('/')
  return parts[parts.length - 1] as string
}

export const toBytes = (hex: string) => {
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

export const ownerFromPublicKey = async (publicKey: string) => {
  const publicKeyBytes = toBytes(publicKey)
  const typeNameBytes = new TextEncoder().encode('Ed25519PublicKey::')
  const bytes = new Uint8Array([...typeNameBytes, ...publicKeyBytes])
  return await keccak(bytes, 256)
}

export const BALANCES = gql`
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

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const getBalances = async () => {
  if (!publicKey.value) return
  const owner = await ownerFromPublicKey(publicKey.value)
  // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access
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

const getProviderState = () => {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access
  window.linera.request({
    method: 'metamask_getProviderState'
  }).then(async (result) => {
    chainId.value = ((result as Record<string, string>).chainId)?.substring(2) as string
    publicKey.value = ((result as Record<string, string>).accounts)?.[0] as string
    await getBalances()
  }).catch((e) => {
    console.log('metamask_getProviderState', e)
  })
}

onMounted(async () => {
  await getChains()
  _applicationId.value = await applicationId()

  getProviderState()

  const _metadata = await metadata(defaultChainUrl.value)

  _balance.value = _metadata.balance
  _totalSupply.value = _metadata.totalSupply

  await linera.initialize()

  void myBalance()
})

</script>
