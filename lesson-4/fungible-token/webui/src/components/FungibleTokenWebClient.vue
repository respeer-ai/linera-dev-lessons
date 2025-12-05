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
import * as linera from '@linera/client'

const chain1Balance = ref('0')
const chain2Balance = ref('0')

const chains = ref([] as string[])
const _applicationId = ref('')

const chain1Url = computed(() => `http://10.1.24.80:8080/chains/${chains.value[0]}/applications/${_applicationId.value}`)
const chain2Url = computed(() => `http://10.1.24.80:8080/chains/${chains.value[1]}/applications/${_applicationId.value}`)

const myBalance = async (url: string) => {
  const resp = await axios.post(url, {
    'query': 'query { myBalance }'
  })
  const data = (resp.data as Record<string, unknown>).data
  return (data as Record<string, string>).myBalance as string
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

const initializeWallet = async () => {
  const faucet = new linera.Faucet(
    'https://faucet.testnet-conway.linera.net',
  );
  const wallet = await faucet.createWallet();
  const signer = linera.PrivateKeySigner.createRandom()
  const client = new linera.Client(wallet, signer, false);

  const owner = signer.address()

  try {
    console.log(`Claiming chain for ${owner} ...`)
    chains.value[0] = await faucet.claimChain(wallet, owner)
    console.log(`Claimed chain for ${owner}`)
    console.log(`Claiming chain for ${owner} ...`)
    chains.value[1] = await faucet.claimChain(wallet, owner)
    console.log(`Claimed chain for ${owner}`)
  } catch (e) {
    console.log(`Failed claim chain:`, e)
  }

  client.onNotification((notification: Record<string, Record<string, unknown>>) => {
    console.log(notification)
  });
}

onMounted(async () => {
  await linera.initialize()
  await initializeWallet()
})

</script>
