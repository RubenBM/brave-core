/* Copyright (c) 2020 The Brave Authors. All rights reserved.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "bat/ledger/internal/endpoint/payment/post_votes/post_votes.h"

#include <utility>

#include "base/base64.h"
#include "base/json/json_writer.h"
#include "base/strings/stringprintf.h"
#include "bat/ledger/internal/credentials/credentials_util.h"
#include "bat/ledger/internal/endpoint/payment/payment_util.h"
#include "bat/ledger/internal/ledger_impl.h"
#include "net/http/http_status_code.h"

using std::placeholders::_1;

namespace ledger {
namespace endpoint {
namespace payment {

PostVotes::PostVotes(bat_ledger::LedgerImpl* ledger):
    ledger_(ledger) {
  DCHECK(ledger_);
}

PostVotes::~PostVotes() = default;

std::string PostVotes::GetUrl() {
  return GetServerUrl("/v1/votes");
}

std::string PostVotes::GeneratePayload(
    const braveledger_credentials::CredentialsRedeem& redeem) {
  base::Value data(base::Value::Type::DICTIONARY);
  data.SetStringKey(
      "type",
      braveledger_credentials::ConvertRewardTypeToString(redeem.type));
  if (!redeem.order_id.empty()) {
    data.SetStringKey("orderId", redeem.order_id);
  }
  data.SetStringKey("channel", redeem.publisher_key);

  std::string data_json;
  base::JSONWriter::Write(data, &data_json);
  std::string data_encoded;
  base::Base64Encode(data_json, &data_encoded);

  base::Value credentials(base::Value::Type::LIST);
  braveledger_credentials::GenerateCredentials(
      redeem.token_list,
      data_encoded,
      &credentials);

  base::Value payload(base::Value::Type::DICTIONARY);
  payload.SetStringKey("vote", data_encoded);
  payload.SetKey("credentials", std::move(credentials));

  std::string json;
  base::JSONWriter::Write(payload, &json);
  return json;
}

ledger::Result PostVotes::CheckStatusCode(const int status_code) {
  if (status_code == net::HTTP_BAD_REQUEST) {
    BLOG(0, "Invalid request");
    return ledger::Result::RETRY_SHORT;
  }

  if (status_code == net::HTTP_INTERNAL_SERVER_ERROR) {
    BLOG(0, "Internal server error");
    return ledger::Result::RETRY_SHORT;
  }

  if (status_code != net::HTTP_OK) {
    return ledger::Result::LEDGER_ERROR;
  }

  return ledger::Result::LEDGER_OK;
}

void PostVotes::Request(
    const braveledger_credentials::CredentialsRedeem& redeem,
    PostVotesCallback callback) {
  auto url_callback = std::bind(&PostVotes::OnRequest,
      this,
      _1,
      callback);
  ledger_->LoadURL(
      GetUrl(),
      {},
      GeneratePayload(redeem),
      "application/json; charset=utf-8",
      ledger::UrlMethod::POST,
      url_callback);
}

void PostVotes::OnRequest(
    const ledger::UrlResponse& response,
    PostVotesCallback callback) {
  ledger::LogUrlResponse(__func__, response);
  callback(CheckStatusCode(response.status_code));
}

}  // namespace payment
}  // namespace endpoint
}  // namespace ledger
