import {
  useAuth,
  useLoggedInServerState,
  useMutableServerState
} from "@/hooks";
import { USER_MUTATIONS, USER_QUERIES } from "@/queries";
import { Box, Center } from "@chakra-ui/layout";
import { DidInfo, GetDIDResponse, TESTNET_CHAIN_NAME } from "@coredin/shared";
import { useContext, useEffect, useState } from "react";
import { RequireWalletConnection } from "../../home/components";
import { Spinner } from "@chakra-ui/spinner";
import { CoredinClientContext } from "@/contexts/CoredinClientContext";
import { useChain } from "@cosmos-kit/react";
import { Navigate, useSearchParams } from "react-router-dom";
import { NotRegisteredProfile } from ".";
import { ROUTES } from "@/router/routes";

export const Profile = () => {
  const chainContext = useChain(TESTNET_CHAIN_NAME);
  const [queryParams] = useSearchParams();
  const coredinClient = useContext(CoredinClientContext);
  const { needsAuth } = useAuth();
  const { data: userProfile, isLoading } = useLoggedInServerState(
    USER_QUERIES.getUser(chainContext.address || "", needsAuth),
    {
      enabled: !!chainContext.address
    }
  );
  const { mutateAsync: updateProfile } = useMutableServerState(
    USER_MUTATIONS.updateProfile()
  );
  const [onchainProfile, setOnchainProfile] = useState<DidInfo | null>(null);
  const [usernameInput, setUsernameInput] = useState<string>(
    userProfile?.username || ""
  );
  const [isRegistering, setIsRegistering] = useState(false);

  const updateOnchainProfile = () => {
    if (chainContext.address) {
      console.log("getting onchain profile");
      coredinClient
        ?.getWalletDID({ wallet: chainContext.address })
        .then((registered_did: GetDIDResponse) => {
          console.log(registered_did);
          if (registered_did.did_info) {
            setOnchainProfile(registered_did.did_info);
          }
        });
    } else {
      setOnchainProfile(null);
    }
  };

  useEffect(updateOnchainProfile, [
    chainContext.address,
    chainContext.isWalletConnected,
    coredinClient
  ]);

  const registerProfile = () => {
    if (
      userProfile &&
      (onchainProfile === null || onchainProfile?.did !== userProfile.did) &&
      usernameInput.length > 2
    ) {
      setIsRegistering(true);
      console.log("Registering profile onchain...", userProfile.did);
      coredinClient
        ?.register({
          did: userProfile.did,
          username: usernameInput
        })
        .then((result) => {
          console.log(result);
          updateOnchainProfile();
          // This forces the backend to retrieve the new username from the blockchain
          updateProfile({
            profile: {}
          });
          setIsRegistering(false);
        })
        .catch((error) => {
          console.log("error while registering");
          console.error(error);
          updateOnchainProfile();
          setIsRegistering(false);
        });
    }
  };

  const handleChangeUserName = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    // Check if the input contains only letters (uppercase and lowercase)
    if ((/^[a-zA-Z0-9]+$/.test(value) || value === "") && value.length <= 64) {
      setUsernameInput(value);
    }
  };
  console.log("loading", isLoading);
  console.log("onchainProfile", onchainProfile);
  return (
    <Box>
      {!onchainProfile && isLoading && (
        <Center mt="32px">
          <Spinner size="xl" color="brand.500" />
        </Center>
      )}
      {!chainContext.address && <RequireWalletConnection />}
      {userProfile &&
        (!onchainProfile || onchainProfile?.did !== userProfile.did) && (
          <NotRegisteredProfile
            did={userProfile.did}
            handleChangeUserName={handleChangeUserName}
            usernameInput={usernameInput}
            registerProfile={registerProfile}
            isRegistering={isRegistering}
          />
        )}
      {onchainProfile &&
        chainContext.isWalletConnected &&
        userProfile &&
        onchainProfile.did === userProfile.did && (
          <Navigate
            to={
              queryParams.get("redirect") ??
              (userProfile.username === "no_username"
                ? ROUTES.SETTINGS.path
                : ROUTES.HOME.path)
            }
          />
        )}
    </Box>
  );
};
