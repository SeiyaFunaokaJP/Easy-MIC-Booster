---
title: 사용자 가이드
layout: default
parent: 홈
nav_order: 1
---

# 사용자 가이드
{: .no_toc }

## 목차
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## 시스템 요구 사항 {#system-requirements}

- **OS**: Windows 10 / 11 (x64, x86 또는 ARM64)
- **가상 오디오 드라이버**: [VB-CABLE](https://vb-audio.com/Cable/) 또는 동등한 가상 케이블

Linux 및 macOS 빌드도 배포되지만 실험적입니다. 해당 환경에서 오디오를 라우팅하는 방법은 [개발자 가이드](developer-guide#supported-platforms)를 참고하세요.

## 설치 {#installation}

### 1. VB-CABLE 설치 {#1-install-vb-cable}

1. [vb-audio.com/Cable](https://vb-audio.com/Cable/)에서 다운로드합니다.
2. 압축을 풀고 `VBCABLE_Setup_x64.exe`를 **관리자 권한으로** 실행합니다.
3. PC를 다시 시작합니다.

### 2. Easy MIC Booster 설치 {#2-install-easy-mic-booster}

[Releases](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest) 페이지에서 최신 릴리스를 다운로드합니다. 다음 중 하나를 선택하세요.

- **설치 프로그램** (`EasyMICBooster-x64-vX.Y.Z-setup.exe`): 관리자 권한 없이 현재 사용자용으로 설치되며 시작 메뉴에 항목이 추가됩니다. 제거는 Windows 설정 > 앱에서 할 수 있으며, 설정은 유지됩니다.
- **포터블** (`EasyMICBooster-x64-vX.Y.Z-portable.zip`): 원하는 폴더에 압축을 풀고 `EasyMICBooster.exe`를 실행합니다.

Windows on ARM에서는 `arm64` 파일을, 32비트 Windows에서만 `x86` 파일을 사용하세요. ARM64 빌드에는 *고품질* 노이즈 억제 방식이 포함되어 있지 않습니다.

## 초기 설정 {#first-time-setup}

### 장치 선택 {#choose-the-devices}

왼쪽의 **라우팅** 카드에서:

| 항목 | 선택 |
|:------|:-------|
| **입력(마이크)** | 사용하는 실제 마이크, 또는 Windows 설정을 따르는 *기본 장치* |
| **출력(가상 케이블)** | `CABLE Input (VB-Audio Virtual Cable)` |

각 목록 아래의 미터는 처리 전후의 레벨을 보여 줍니다. 두 미터 사이의 줄은 현재 활성화된 처리를 요약합니다(예: `+12.0 dB · NS · Gate`).

{: .warning }
출력이 스피커나 헤드폰(기본 재생 장치)이면 자신의 목소리가 들리게 됩니다. 출력은 반드시 가상 케이블을 선택하세요.

### 게인 설정 {#set-the-gain}

1. 오른쪽 위의 **BOOST ON**이 켜져 있는지 확인합니다.
2. 평소처럼 말하면서 **사운드** 카드의 **게인**을 출력 미터의 피크가 초록~노랑 영역에 오도록 올립니다. −1 / +1 버튼은 1 dB씩 조정합니다.
3. 미터가 빨갛게 변하고 *리미터*가 표시되면 리미터가 피크를 억누르고 있는 것입니다. 게인을 조금 낮추세요.

오디오가 흐르고 있으면 제목 옆의 상태 표시에 샘플 레이트 및 지연 시간과 함께 **작동 중**이 표시됩니다.

### 수신 앱 설정 {#configure-the-receiving-app}

Discord, Zoom, OBS, Teams 등에서 오디오 설정을 열고 마이크(입력 장치)로 `CABLE Output (VB-Audio Virtual Cable)`을 선택합니다.

{: .warning }
Windows 사운드 설정에서 `CABLE Output`을 **기본 재생 장치**로 설정하지 **마세요**. 피드백 루프(하울링)가 발생합니다.

## 일상적인 사용 {#daily-use}

- **바이패스** -- **BOOST ON**을 끄면 게인과 EQ 없이 마이크 소리를 그대로 통과시킵니다(노이즈 억제와 게이트는 설정대로 유지됩니다).
- **트레이** -- 창을 닫아도 앱은 알림 영역에서 계속 실행됩니다. 트레이 아이콘을 우클릭하면 부스트나 노이즈 억제 켜기/끄기, 게인 변경, 프리셋·장치 프로필·장치 전환, 종료를 할 수 있습니다. 클릭하면 창이 열립니다.
- **프리셋** -- **새로 저장**으로 사운드를 저장하고 **프리셋** 카드나 트레이에서 프리셋을 전환합니다. [기능](features#presets)을 참고하세요.
- **장치 프로필** -- 마이크/출력 조합을 저장하고 전환합니다(예: 책상에서는 헤드셋, 방송할 때는 USB 마이크). [기능](features#device-profiles)을 참고하세요.
- **장치 분리** -- 마이크나 출력 장치가 사라지면 상태가 빨갛게 표시되고, 다시 연결되면 라우팅이 자동으로 재개됩니다.

## 설정 {#settings}

오른쪽 위의 톱니바퀴 아이콘으로 **설정**을 엽니다.

| 설정 | 설명 |
|:--------|:------------|
| **언어** | UI 언어(처음 실행 시 Windows 설정을 따름) |
| **로그인 시 시작** | Windows에 로그인하면 자동으로 실행(트레이에서 시작) |
| **창을 닫아도 트레이에서 계속 실행** | 끄면: 창을 닫을 때 앱이 종료됨 |
| **+50 dB 초과 게인 허용** | 게인 상한을 +100 dB로 높임 |
| **출력 상한** | 리미터가 출력을 이 피크 레벨 아래로 유지(기본값 −1.0 dBFS) |
| **업데이트 확인** | 시작 시 GitHub에서 새 릴리스를 확인 |

아래 버튼으로 설정 폴더, GitHub 페이지, 후원 페이지를 열거나 앱을 종료할 수 있습니다.

설정, 프리셋, 장치 프로필은 `%APPDATA%\EasyMICBooster`에 자동으로 저장되며 다음 실행 시 복원됩니다. v1.x의 설정은 처음 시작할 때 가져옵니다.

모든 오디오 컨트롤에 대한 자세한 내용은 [기능](features)을 참고하세요.
