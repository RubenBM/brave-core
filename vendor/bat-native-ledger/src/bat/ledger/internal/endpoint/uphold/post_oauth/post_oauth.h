/* Copyright (c) 2020 The Brave Authors. All rights reserved.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef BRAVELEDGER_ENDPOINT_UPHOLD_POST_OAUTH_POST_OAUTH_H_
#define BRAVELEDGER_ENDPOINT_UPHOLD_POST_OAUTH_POST_OAUTH_H_

#include <string>

#include "bat/ledger/ledger.h"

// POST https://api.uphold.com/oauth2/token
//
// Request body:
// code=wewfwkfpkwpfkwofkwpofk&grant_type=authorization_code
//
// Success code:
// HTTP_OK (200)
//
// Error codes:
// HTTP_UNAUTHORIZED (401)
//
// Response body:
// {
//   "access_token": "edc8b465fe2e2a26ce553d937ccc6c7195e9f909",
//   "token_type": "bearer",
//   "expires_in": 7775999,
//   "scope": "accounts:read accounts:write cards:read cards:write user:read"
// }

namespace bat_ledger {
class LedgerImpl;
}

namespace ledger {
namespace endpoint {
namespace uphold {

using PostOauthCallback = std::function<void(
    const ledger::Result result,
    const std::string& token)>;

class PostOauth {
 public:
  explicit PostOauth(bat_ledger::LedgerImpl* ledger);
  ~PostOauth();

  void Request(
      const std::string& code,
      PostOauthCallback callback);

 private:
  std::string GetUrl();

  std::string GeneratePayload(const std::string& code);

  ledger::Result CheckStatusCode(const int status_code);

  ledger::Result ParseBody(
      const std::string& body,
      std::string* token);

  void OnRequest(
      const ledger::UrlResponse& response,
      PostOauthCallback callback);

  bat_ledger::LedgerImpl* ledger_;  // NOT OWNED
};

}  // namespace uphold
}  // namespace endpoint
}  // namespace ledger

#endif  // BRAVELEDGER_ENDPOINT_UPHOLD_POST_OAUTH_POST_OAUTH_H_
