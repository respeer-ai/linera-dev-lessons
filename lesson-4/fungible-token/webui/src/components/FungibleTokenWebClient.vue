<template>
  <div>
    <p class='text-h1'>Fungible Token</p>
    <p class='text-h6'>Faucet Chain: {{ faucetChain }}</p>
    <p class='text-h6'>Default Chain: {{ defaultChain }}</p>
    <p class='text-h6'>Application: {{ _applicationId }}</p>
    <p class='text-h6'>Total Supply: {{ _totalSupply }}</p>
    <p class='text-h6'>Supply Balance: {{ _balance }}</p>
    <p class='text-h6'>From: {{ wallet1?.chain }}@{{ wallet1?.owner }} {{ wallet1?.balance }}</p>
    <p class='text-h6'>To: {{ wallet2?.chain }}@{{ wallet2?.owner }} {{ wallet2?.balance }}</p>
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

const _balance = ref('0')
const _totalSupply = ref('0')

const _applicationId = ref('')
const defaultChain = ref('')
const faucetChain = ref('8fd4233c5d03554f87d47a711cf70619727ca3d148353446cab81fb56922c9b7')
const defaultChainUrl = computed(() => `http://192.168.31.182:8080/chains/${defaultChain.value}/applications/${_applicationId.value}`)

interface Wallet {
  chain: string
  owner: string
  application: linera.Application
  balance?: string
}

const wallet1 = ref(undefined as unknown as Wallet)
const wallet2 = ref(undefined as unknown as Wallet)

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
        mint(amount: "1.235")
      }
    `
  }

  await wallet1.value.application.query(JSON.stringify(query))
  await wallet2.value.application.query(JSON.stringify(query))
}

const onTransferClick = async () => {
  let query = {
    query: `
      mutation {
        transfer(to: "${wallet2.value.chain}", amount: "1.235")
      }
    `
  }

  await wallet1.value.application.query(JSON.stringify(query))

  query = {
    query: `
      mutation {
        transfer(to: "${wallet1.value.chain}", amount: "0.0001")
      }
    `
  }

  await wallet2.value.application.query(JSON.stringify(query))
}

const onRefreshClick = () => {
  void myBalance(wallet1.value, () => {
    void myBalance(wallet2.value, () => {}, () => {})
  }, () => {})
}

const initializeWallet = async (onNewBlock: (hash: string) => void) => {
  const faucet = new linera.Faucet(
    // 'https://faucet.testnet-conway.linera.net'
    'https://api.testnet-conway.faucet.respeer.ai/api/faucet'
  );
  const wallet = await faucet.createWallet();
  const signer = new metamask.Signer()

  const owner = await signer.address()
  const chain = await faucet.claimChain(wallet, owner)

  // eslint-disable-next-line @typescript-eslint/await-thenable
  const client = await new linera.Client(wallet, signer, false);
  const application = await client.application(_applicationId.value)

  client.onNotification((notification: Record<string, Record<string, Record<string, string>>>) => {
    if (notification.reason?.NewBlock) {
      onNewBlock(notification.reason?.NewBlock?.hash as string)
    }
  })

  return {
    owner,
    chain,
    application
  }
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

const myBalance = (wallet: Wallet, done: () => void, error: () => void) => {
  console.log(`Query balance of ${wallet.chain}`)

  wallet.application.query('{"query": "query { myBalance }"}').then((respStr) => {
    const resp = JSON.parse(respStr)
    wallet.balance = resp.data.myBalance
    done()
  }).catch((e) => {
    console.log('Failed query:', e)
    error()
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

onMounted(async () => {
  await getChains()
  _applicationId.value = await applicationId()

  const _metadata = await metadata(defaultChainUrl.value)

  _balance.value = _metadata.balance
  _totalSupply.value = _metadata.totalSupply

  await linera.initialize()

  wallet1.value = await initializeWallet(() => {
    void myBalance(wallet1.value, () => {}, () => {})
  })
  wallet2.value = await initializeWallet(() => {
    void myBalance(wallet2.value, () => {}, () => {})
  })

  void myBalance(wallet1.value, () => {
    void myBalance(wallet2.value, () => {}, () => {})
  }, () => {})
})

</script>
