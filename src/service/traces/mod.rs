// Copyright 2023 Zinc Labs Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::io::Error;

use actix_web::{http, HttpResponse};
use ahash::AHashMap;
use bytes::BytesMut;
use chrono::{Duration, Utc};
use datafusion::arrow::datatypes::Schema;
use opentelemetry::trace::{SpanId, TraceId};
use opentelemetry_proto::tonic::{
    collector::trace::v1::{
        ExportTracePartialSuccess, ExportTraceServiceRequest, ExportTraceServiceResponse,
    },
    trace::v1::{status::StatusCode, Status},
};
use prost::Message;

use crate::{
    common::{
        infra::{
            cluster,
            config::{CONFIG, DISTINCT_FIELDS},
            metrics,
        },
        meta::{
            alerts::Alert,
            http::HttpResponse as MetaHttpResponse,
            stream::{PartitionTimeLevel, SchemaRecords, StreamParams},
            traces::{Event, Span, SpanRefType},
            usage::UsageType,
            StreamType,
        },
        utils::{self, flatten, hasher::get_fields_key_xxh3, json},
    },
    service::{
        db, distinct_values, format_partition_key, format_stream_name,
        ingestion::{evaluate_trigger, grpc::get_val, write_file_arrow, TriggerAlertData},
        schema::{check_for_schema, stream_schema_exists},
        stream::unwrap_partition_time_level,
        usage::report_request_usage_stats,
    },
};

pub mod otlp_http;

const PARENT_SPAN_ID: &str = "reference.parent_span_id";
const PARENT_TRACE_ID: &str = "reference.parent_trace_id";
const REF_TYPE: &str = "reference.ref_type";
const SERVICE_NAME: &str = "service.name";
const SERVICE: &str = "service";

pub async fn handle_trace_request(
    org_id: &str,
    thread_id: usize,
    request: ExportTraceServiceRequest,
    is_grpc: bool,
    in_stream_name: Option<&str>,
) -> Result<HttpResponse, Error> {
    if !cluster::is_ingester(&cluster::LOCAL_NODE_ROLE) {
        return Ok(
            HttpResponse::InternalServerError().json(MetaHttpResponse::error(
                http::StatusCode::INTERNAL_SERVER_ERROR.into(),
                "not an ingester".to_string(),
            )),
        );
    }

    if !db::file_list::BLOCKED_ORGS.is_empty() && db::file_list::BLOCKED_ORGS.contains(&org_id) {
        return Ok(HttpResponse::Forbidden().json(MetaHttpResponse::error(
            http::StatusCode::FORBIDDEN.into(),
            "Quota exceeded for this organization".to_string(),
        )));
    }
    let start = std::time::Instant::now();

    let traces_stream_name = match in_stream_name {
        Some(name) => format_stream_name(name),
        None => "default".to_owned(),
    };

    let traces_stream_name = &traces_stream_name;

    let mut runtime = crate::service::ingestion::init_functions_runtime();
    let mut stream_alerts_map: AHashMap<String, Vec<Alert>> = AHashMap::new();
    let mut traces_schema_map: AHashMap<String, Schema> = AHashMap::new();
    let mut distinct_values = Vec::with_capacity(16);

    let stream_schema = stream_schema_exists(
        org_id,
        traces_stream_name,
        StreamType::Traces,
        &mut traces_schema_map,
    )
    .await;

    let mut partition_keys: Vec<String> = vec![];
    let mut partition_time_level =
        PartitionTimeLevel::from(CONFIG.limit.traces_file_retention.as_str());
    if stream_schema.has_partition_keys {
        let partition_det = crate::service::ingestion::get_stream_partition_keys(
            traces_stream_name,
            &traces_schema_map,
        )
        .await;
        partition_keys = partition_det.partition_keys;
        partition_time_level =
            unwrap_partition_time_level(partition_det.partition_time_level, StreamType::Traces);
    }

    // Start get stream alerts
    crate::service::ingestion::get_stream_alerts(
        org_id,
        StreamType::Traces,
        traces_stream_name,
        &mut stream_alerts_map,
    )
    .await;
    // End get stream alert

    // Start Register Transforms for stream
    let (local_trans, stream_vrl_map) = crate::service::ingestion::register_stream_transforms(
        org_id,
        StreamType::Traces,
        traces_stream_name,
    );
    // End Register Transforms for stream

    let mut trigger: TriggerAlertData = None;

    let min_ts =
        (Utc::now() - Duration::hours(CONFIG.limit.ingest_allowed_upto)).timestamp_micros();
    let mut partial_success = ExportTracePartialSuccess::default();

    let mut data_buf: AHashMap<String, SchemaRecords> = AHashMap::new();

    let mut service_name: String = traces_stream_name.to_string();
    let res_spans = request.resource_spans;
    for res_span in res_spans {
        let mut service_att_map: AHashMap<String, json::Value> = AHashMap::new();
        let resource = res_span.resource.unwrap();

        for res_attr in resource.attributes {
            if res_attr.key.eq(SERVICE_NAME) {
                let loc_service_name = get_val(&res_attr.value.as_ref());
                if !loc_service_name.eq(&json::Value::Null) {
                    service_name = loc_service_name.as_str().unwrap().to_string();
                    service_att_map.insert(res_attr.key, loc_service_name);
                }
            } else {
                service_att_map.insert(
                    format!("{}.{}", SERVICE, res_attr.key),
                    get_val(&res_attr.value.as_ref()),
                );
            }
        }
        let inst_resources = res_span.scope_spans;
        for inst_span in inst_resources {
            let spans = inst_span.spans;
            for span in spans {
                let span_id: String = SpanId::from_bytes(
                    span.span_id
                        .as_slice()
                        .try_into()
                        .expect("slice with incorrect length"),
                )
                .to_string();
                let trace_id: String = TraceId::from_bytes(
                    span.trace_id
                        .as_slice()
                        .try_into()
                        .expect("slice with incorrect length"),
                )
                .to_string();
                let mut span_ref = AHashMap::new();
                if !span.parent_span_id.is_empty() {
                    span_ref.insert(PARENT_TRACE_ID.to_string(), trace_id.clone());
                    span_ref.insert(
                        PARENT_SPAN_ID.to_string(),
                        SpanId::from_bytes(
                            span.parent_span_id
                                .as_slice()
                                .try_into()
                                .expect("slice with incorrect length"),
                        )
                        .to_string(),
                    );
                    span_ref.insert(REF_TYPE.to_string(), format!("{:?}", SpanRefType::ChildOf));
                }
                let start_time: u64 = span.start_time_unix_nano;
                let end_time: u64 = span.end_time_unix_nano;
                let mut span_att_map: AHashMap<String, json::Value> = AHashMap::new();
                for span_att in span.attributes {
                    span_att_map.insert(span_att.key, get_val(&span_att.value.as_ref()));
                }

                let mut events = vec![];
                let mut event_att_map: AHashMap<String, json::Value> = AHashMap::new();
                for event in span.events {
                    for event_att in event.attributes {
                        event_att_map.insert(event_att.key, get_val(&event_att.value.as_ref()));
                    }
                    events.push(Event {
                        name: event.name,
                        _timestamp: event.time_unix_nano,
                        attributes: event_att_map.clone(),
                    })
                }

                let timestamp = start_time / 1000;

                let local_val = Span {
                    trace_id: trace_id.clone(),
                    span_id,
                    span_kind: span.kind.to_string(),
                    span_status: get_span_status(span.status),
                    operation_name: span.name.clone(),
                    start_time,
                    end_time,
                    duration: (end_time - start_time) / 1000, // microseconds
                    reference: span_ref,
                    service_name: service_name.clone(),
                    attributes: span_att_map,
                    service: service_att_map.clone(),
                    flags: 1, // TODO add appropriate value
                    //_timestamp: timestamp,
                    events: json::to_string(&events).unwrap(),
                };

                let value: json::Value = json::to_value(local_val).unwrap();

                // JSON Flattening
                let mut value = flatten::flatten(&value).unwrap();

                if !local_trans.is_empty() {
                    value = crate::service::ingestion::apply_stream_transform(
                        &local_trans,
                        &value,
                        &stream_vrl_map,
                        traces_stream_name,
                        &mut runtime,
                    )
                    .unwrap_or(value);
                }
                // End row based transform */
                // get json object
                let val_map = value.as_object_mut().unwrap();

                val_map.insert(
                    CONFIG.common.column_timestamp.clone(),
                    json::Value::Number(timestamp.into()),
                );

                // get distinct_value item
                for field in DISTINCT_FIELDS.iter() {
                    if let Some(val) = val_map.get(field) {
                        if !val.is_null() {
                            let (filter_name, filter_value) = if field == "operation_name" {
                                ("service_name".to_string(), service_name.clone())
                            } else {
                                ("".to_string(), "".to_string())
                            };
                            distinct_values.push(distinct_values::DvItem {
                                stream_type: StreamType::Traces,
                                stream_name: traces_stream_name.to_string(),
                                field_name: field.to_string(),
                                field_value: val.as_str().unwrap().to_string(),
                                filter_name,
                                filter_value,
                            });
                        }
                    }
                }

                let value_str = crate::common::utils::json::to_string(&val_map).unwrap();

                // check schema
                let schema_evolution = check_for_schema(
                    org_id,
                    traces_stream_name,
                    StreamType::Traces,
                    &value_str,
                    &mut traces_schema_map,
                    timestamp.try_into().unwrap(),
                    true,
                )
                .await;

                // get hour key
                let schema_key = get_fields_key_xxh3(&schema_evolution.schema_fields);

                // get hour key
                let mut hour_key = super::ingestion::get_wal_time_key(
                    timestamp.try_into().unwrap(),
                    &partition_keys,
                    partition_time_level,
                    val_map,
                    Some(&schema_key),
                );

                if trigger.is_none() && !stream_alerts_map.is_empty() {
                    // Start check for alert trigger
                    let key = format!("{}/{}/{}", &org_id, StreamType::Traces, traces_stream_name);
                    if let Some(alerts) = stream_alerts_map.get(&key) {
                        let mut trigger_alerts: Vec<(Alert, Vec<json::Map<String, json::Value>>)> =
                            Vec::new();
                        for alert in alerts {
                            if let Ok(Some(v)) = alert.evaluate(Some(val_map)).await {
                                trigger_alerts.push((alert.clone(), v));
                            }
                        }
                        trigger = Some(trigger_alerts);
                    }
                    // End check for alert trigger
                }

                if partition_keys.is_empty() {
                    let partition_key = format!("service_name={}", service_name);
                    hour_key.push_str(&format!("/{}", format_partition_key(&partition_key)));
                }
                let rec_schema = traces_schema_map.get(traces_stream_name).unwrap();

                let hour_buf = data_buf.entry(hour_key).or_insert(SchemaRecords {
                    schema: rec_schema
                        .clone()
                        .with_metadata(std::collections::HashMap::new()),
                    records: vec![],
                });
                let loc_value: utils::json::Value =
                    utils::json::from_slice(value_str.as_bytes()).unwrap();
                hour_buf.records.push(loc_value);

                if timestamp < min_ts.try_into().unwrap() {
                    partial_success.rejected_spans += 1;
                    continue;
                }
            }
        }
    }

    let mut traces_file_name = "".to_string();
    let mut req_stats = write_file_arrow(
        &data_buf,
        thread_id,
        &StreamParams::new(org_id, traces_stream_name, StreamType::Traces),
        &mut traces_file_name,
        None,
    )
    .await;
    let time = start.elapsed().as_secs_f64();
    req_stats.response_time = time;

    // send distinct_values
    if !distinct_values.is_empty() {
        if let Err(e) = distinct_values::write(org_id, distinct_values).await {
            log::error!("Error while writing distinct values: {}", e);
        }
    }

    let ep = if is_grpc {
        "grpc/export/traces"
    } else {
        "http-proto/api/org/traces/"
    };
    metrics::HTTP_RESPONSE_TIME
        .with_label_values(&[
            ep,
            "200",
            org_id,
            traces_stream_name,
            StreamType::Traces.to_string().as_str(),
        ])
        .observe(time);
    metrics::HTTP_INCOMING_REQUESTS
        .with_label_values(&[
            ep,
            "200",
            org_id,
            traces_stream_name,
            StreamType::Traces.to_string().as_str(),
        ])
        .inc();

    // metric + data usage
    report_request_usage_stats(
        req_stats,
        org_id,
        traces_stream_name,
        StreamType::Traces,
        UsageType::Traces,
        0,
    )
    .await;

    // only one trigger per request, as it updates etcd
    evaluate_trigger(trigger).await;

    let res = ExportTraceServiceResponse {
        partial_success: if partial_success.rejected_spans > 0 {
            partial_success.error_message =
                "Some spans were rejected due to exceeding the allowed retention period"
                    .to_string();
            Some(partial_success)
        } else {
            None
        },
    };
    let mut out = BytesMut::with_capacity(res.encoded_len());
    res.encode(&mut out).expect("Out of memory");

    return Ok(HttpResponse::Ok()
        .status(http::StatusCode::OK)
        .content_type("application/x-protobuf")
        .body(out));
}

fn get_span_status(status: Option<Status>) -> String {
    match status {
        Some(v) => match v.code() {
            StatusCode::Ok => "OK".to_string(),
            StatusCode::Error => "ERROR".to_string(),
            StatusCode::Unset => "UNSET".to_string(),
        },
        None => "".to_string(),
    }
}
