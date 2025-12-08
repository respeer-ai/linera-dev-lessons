import { graphqlResult } from 'src/utils/index'
import { getClientOptions } from 'src/apollo'
import { ApolloClient } from '@apollo/client'
import { provideApolloClient, useSubscription } from '@vue/apollo-composable'

import { gql } from '@apollo/client/core'

export type NotificationsSubscription = { __typename?: 'SubscriptionRoot', notifications: unknown };

export const BALANCES = gql`
  query balances($chainOwners: [ChainOwners!]!) {
    balances(chainOwners: $chainOwners)
  }
`

export const NOTIFICATIONS = gql`
  subscription notifications($chainId: ChainId!) {
    notifications(chainId: $chainId)
  }
`

export class Subscription {
  #unsubscribe: () => void = undefined as unknown as () => void

  constructor(
    httpUrl: string,
    wsUrl: string,
    chainId: string,
    onNewBlock: (hash: string) => void
  ) {
    const options = /* await */ getClientOptions(httpUrl, wsUrl)
    const apolloClient = new ApolloClient(options)

    const { /* result, refetch, fetchMore, */ stop, onResult, onError } =
      provideApolloClient(apolloClient)(() =>
        useSubscription(NOTIFICATIONS, {
          chainId
        })
      )

    onError((error) => {
      // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
      console.log(`Fail subscribe to ${chainId}: ${error}`)
    })

    onResult((res) => {
      const notifications = (
        graphqlResult.rootData(res) as NotificationsSubscription
      ).notifications
      const reason = graphqlResult.keyValue(notifications, 'reason')
      const newBlock = graphqlResult.keyValue(reason, 'NewBlock')
      if (newBlock) {
        onNewBlock?.(graphqlResult.keyValue(newBlock, 'hash') as string)
      }
    })

    this.#unsubscribe = stop
  }

  unsubscribe = this.#unsubscribe
}
