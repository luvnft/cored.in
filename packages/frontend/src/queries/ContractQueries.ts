import { BaseServerStateKeys } from "@/constants";
import { CoredinClient } from "@coredin/shared";

const PAGE_SIZE = 10;

export const CONTRACT_QUERIES = {
  getWalletDid: (coredinClient: CoredinClient, wallet: string) => ({
    queryKey: [BaseServerStateKeys.CONTRACT_GET_WALLET_DID, wallet],
    queryFn: () => coredinClient.getWalletDID({ wallet })
  }),
  getInfoFromDid: (coredinClient: CoredinClient, did: string) => ({
    queryKey: [BaseServerStateKeys.CONTRACT_GET_DID_INFO, did],
    queryFn: () => coredinClient.getDID({ did })
  }),
  getSubscriptionPrice: (coredinClient: CoredinClient, did: string) => ({
    queryKey: [BaseServerStateKeys.CONTRACT_GET_SUBSCRIPTION_PRICE, did],
    queryFn: () => coredinClient.getSubscriptionPrice({ did })
  }),
  getSubscriptionDuration: (coredinClient: CoredinClient, did: string) => ({
    queryKey: [BaseServerStateKeys.CONTRACT_GET_SUBSCRIPTION_DURATION, did],
    queryFn: () => coredinClient.getSubscriptionDuration({ did })
  }),
  getSubscribers: (coredinClient: CoredinClient, wallet: string) => ({
    queryKey: [
      BaseServerStateKeys.CONTRACT_GET_SUBSCRIBERS,
      wallet,
      coredinClient.contractAddress
    ],
    queryFn: async ({ pageParam }: { pageParam: number }) => {
      console.log(
        "pageParam",
        pageParam,
        "wallet",
        wallet,
        "coredinClient",
        coredinClient
      );
      console.log("PAGE_SIZE.toString()", PAGE_SIZE.toString());
      const res = await coredinClient.getSubscribers({
        wallet,
        page: pageParam.toString(),
        pageSize: PAGE_SIZE.toString()
      });
      console.log("res", res);
      // return res;
      // Quick hack filtering in the frontend since the contract is currently returning all subscribptions alltogether
      return {
        subscribers: res.subscribers.filter(
          (sub) => sub.subscribed_to_wallet === wallet
        )
      };
    }
  }),
  getSubscriptions: (coredinClient: CoredinClient, wallet: string) => ({
    queryKey: [
      BaseServerStateKeys.CONTRACT_GET_SUBSCRIPTIONS,
      wallet,
      coredinClient.contractAddress
    ],
    queryFn: async ({ pageParam }: { pageParam: number }) => {
      console.log(
        "getSubscriptionspageParam",
        pageParam,
        "getSubscriptionswallet",
        wallet,
        "getSubscriptionscoredinClient",
        coredinClient
      );
      console.log("getSubscriptionsPAGE_SIZE.toString()", PAGE_SIZE.toString());
      const res = await coredinClient.getSubscriptions({
        wallet,
        page: pageParam.toString(),
        pageSize: PAGE_SIZE.toString()
      });
      console.log("getSubscriptionsres", res);
      return res;
    }
  }),
  isSubscriber: (
    coredinClient: CoredinClient,
    profileDid: string,
    subscriberWallet: string
  ) => ({
    queryKey: [
      BaseServerStateKeys.CONTRACT_IS_SUBSCRIBED,
      profileDid,
      subscriberWallet
    ],
    queryFn: () =>
      coredinClient.isSubscriber({ targetDid: profileDid, subscriberWallet })
  }),
  getSubscriptionInfo: (
    coredinClient: CoredinClient,
    profileDid: string,
    subscriberWallet: string
  ) => ({
    queryKey: [
      BaseServerStateKeys.CONTRACT_SUBSCRIPTION_INFO,
      profileDid,
      subscriberWallet
    ],
    queryFn: () =>
      coredinClient.getSubscriptionInfo({
        did: profileDid,
        subscriber: subscriberWallet
      })
  })
};
