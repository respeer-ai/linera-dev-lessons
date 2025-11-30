<template>
  <div>
    <p class='text-h1'>Fungible Token</p>
    <p class='text-h6'>From: {{ chains[0] }} {{ chain1Balance }}</p>
    <p class='text-h6'>To: {{ chains[1] }} {{ chain2Balance }}</p>
    <q-btn @click='onMintClick' class='text-h4'>Mint</q-btn>
    <q-btn @click='onTransferClick' class='text-h4'>Transfer</q-btn>
  </div>
</template>

<script setup lang="ts">
import axios from 'axios';
import { computed, onMounted, ref } from 'vue'
import { Subscription } from 'src/subscription'

const chain1Balance = ref('0')
const chain2Balance = ref('0')

const defaultChain = ref('')
const chains = ref([] as string[])
const faucetChain = ref('8fd4233c5d03554f87d47a711cf70619727ca3d148353446cab81fb56922c9b7')
const _applicationId = ref('')

const chain1Url = computed(() => `http://192.168.31.182:8080/chains/${chains.value[0]}/applications/${_applicationId.value}`)
const chain2Url = computed(() => `http://192.168.31.182:8080/chains/${chains.value[1]}/applications/${_applicationId.value}`)

const myBalance = async (url: string) => {
  const resp = await axios.post(url, {
    'query': 'query { myBalance }'
  })
  const data = (resp.data as Record<string, unknown>).data
  return (data as Record<string, string>).myBalance as string
}

const getChains = async () => {
  const url = 'http://192.168.31.182:8080'
  const resp = await axios.post(url, {
    'query': 'query { chains { list default } }'
  })
  const data = (resp.data as Record<string, unknown>).data
  defaultChain.value =  ((data as Record<string, string>).chains as unknown as Record<string, string>).default as string
  chains.value = (((data as Record<string, string>).chains as unknown as Record<string, string>).list as unknown as []).filter((el) => el !== faucetChain.value) as string[]

  const wsUrl = 'ws://192.168.31.182:8080/ws'
  new Subscription(url, wsUrl, chains.value[0] as string, (hash: string) => {
    console.log(`NewBlock ${hash} on chain ${chains.value[0]}`)
    myBalance(chain1Url.value).then((balance) => {
      chain1Balance.value = balance
    })
  })
  new Subscription(url, wsUrl, chains.value[1] as string, (hash: string) => {
    console.log(`NewBlock ${hash} on chain ${chains.value[1]}`)
    myBalance(chain2Url.value).then((balance) => {
      chain2Balance.value = balance
    })
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

const onMintClick = async () => {
  await axios.post(chain1Url.value, {
    'query': 'mutation { mint(amount: "1.23454") }'
  })
  chain1Balance.value = await myBalance(chain1Url.value)
}

const onTransferClick = async () => {
  await axios.post(chain1Url.value, {
    'query': `mutation { transfer(to: "${chains.value[1]}", amount: "1.234253") }`
  })

  chain1Balance.value = await myBalance(chain1Url.value)
  chain2Balance.value = await myBalance(chain2Url.value)
}

onMounted(async () => {
  await getChains()
  _applicationId.value = await applicationId()

  chain1Balance.value = await myBalance(chain1Url.value)
  chain2Balance.value = await myBalance(chain2Url.value)
})

</script>
