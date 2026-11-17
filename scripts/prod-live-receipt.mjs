#!/usr/bin/env node
// 生成 domainx-prod-live E1 机器收据（trust_level=E1，schema v2）。
// v2 相对 v1：脚本内实际执行消费者 example，把 consumer_run（命令、退出码、
// 耗时、stdout 尾部、stdout SHA-256）与 verification.dry_run=false 写入收据，
// 收据自身绑定运行结果，不再依赖 CI job 事后 grep。
// 环境变量缺失或非常规运行属工具错误，退出码 2；HEAD/CI SHA 不一致或消费者
// 运行失败属内容发现，退出码 1（收据仍会先落盘，供 artifacts 归档取证）。
import { writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";

const RECEIPT_PATH = process.env.RECEIPT_OUT || "domainx-live-receipt.json";
const CONSUMER_MANIFEST = "Cargo.toml";
const CONSUMER_EXAMPLE = "domainx_basic";
const CONSUMER_MARKER = "domainx-consumer: ok";
const CONSUMER_PROFILE_MARKER = "profile=production";

function toolError(message) {
  console.error(`prod-live-receipt 工具错误：${message}`);
  process.exit(2);
}

function git(args) {
  const result = spawnSync("git", args, { encoding: "utf8" });
  if (result.error) toolError(`git ${args.join(" ")}: ${result.error.message}`);
  if (result.status !== 0) toolError(`git ${args.join(" ")}: ${(result.stderr || "").trim()}`);
  return result.stdout.trim();
}

const commitSha = process.env.CI_COMMIT_SHA;
if (!commitSha || !/^[0-9a-f]{40}$/.test(commitSha)) {
  toolError("CI_COMMIT_SHA 缺失或不是 40 位十六进制 SHA");
}
if (git(["rev-parse", "HEAD"]) !== commitSha) {
  console.error(`prod-live-receipt 内容发现：HEAD 与 CI_COMMIT_SHA 不一致`);
  process.exit(1);
}

const profile = process.env.DOMAINX_LIVE_PROFILE;
if (!profile) toolError("DOMAINX_LIVE_PROFILE 未设置");

const headTree = git(["rev-parse", "HEAD^{tree}"]);
const consumerCommand = `DOMAINX_LIVE_PROFILE=${profile} cargo run --example ${CONSUMER_EXAMPLE}`;

const started = Date.now();
const run = spawnSync("cargo", ["run", "--example", CONSUMER_EXAMPLE], {
  encoding: "utf8",
  env: { ...process.env, DOMAINX_LIVE_PROFILE: profile },
  maxBuffer: 16 * 1024 * 1024,
});
const elapsedMs = Date.now() - started;
if (run.error) toolError(`cargo run --example ${CONSUMER_EXAMPLE}: ${run.error.message}`);

const stdout = `${run.stdout || ""}`;
const stdoutTail = stdout.trimEnd().split("\n").slice(-5).join("\n");
const consumerRun = {
  command: consumerCommand,
  exit_code: run.status,
  elapsed_ms: elapsedMs,
  stdout_tail: stdoutTail,
  stdout_sha256: createHash("sha256").update(stdout).digest("hex"),
};

const markerOk = run.status === 0 && stdout.includes(CONSUMER_MARKER) && stdout.includes(CONSUMER_PROFILE_MARKER);

const receipt = {
  schema: "domainx-prod-live-receipt/v2",
  trust_level: "E1",
  evidence_id: `E1-domainx-live-${process.env.CI_JOB_NAME || "local"}-${process.env.CI_PIPELINE_ID || "local"}`,
  commit_sha: commitSha,
  tree_sha: headTree,
  pipeline_id: process.env.CI_PIPELINE_ID || null,
  job_id: process.env.CI_JOB_ID || null,
  job_name: process.env.CI_JOB_NAME || null,
  consumer: {
    kind: "example",
    manifest: CONSUMER_MANIFEST,
    example: CONSUMER_EXAMPLE,
    expected_marker: CONSUMER_MARKER,
    profile,
  },
  consumer_run: consumerRun,
  verification: {
    result: markerOk ? "PASS" : "FAIL",
    dry_run: false,
    binding: {
      commit_sha: commitSha,
      tree_sha: headTree,
      environment: "internal-production",
      timestamp: new Date().toISOString(),
      outcome: markerOk ? "PASS" : "FAIL",
    },
  },
  generated_at: new Date().toISOString(),
};

writeFileSync(RECEIPT_PATH, `${JSON.stringify(receipt, null, 2)}\n`);
console.log(`prod-live-receipt: wrote ${RECEIPT_PATH} (commit ${commitSha.slice(0, 9)}, consumer exit=${run.status})`);

if (!markerOk) {
  console.error(
    `prod-live-receipt 内容发现：消费者运行未满足断言（exit=${run.status}, marker=${stdout.includes(CONSUMER_MARKER)}, profile=${stdout.includes(CONSUMER_PROFILE_MARKER)}）`,
  );
  process.exit(1);
}
