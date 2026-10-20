---
title: 홈
layout: default
nav_order: 14
has_children: true
has_toc: false
---

# Easy MIC Booster

Windows용 마이크 게인 증폭 및 고품질 오디오 처리 도구.
{: .fs-6 .fw-300 }

마이크 입력을 받아 노이즈 억제, 이퀄라이저, 게인, 노이즈 게이트, 리미터로 깨끗하게 다듬고 증폭한 뒤, VB-CABLE 같은 가상 오디오 장치로 보냅니다. 그래서 모든 앱이 처리된 목소리를 듣게 됩니다.

[시작하기](user-guide){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[다운로드](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest){: .btn .fs-5 .mb-4 .mb-md-0 .mr-2 }
[GitHub](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster){: .btn .fs-5 .mb-4 .mb-md-0 }

![Easy MIC Booster 메인 창](../images/app_top.png)

---

## 주요 기능 {#key-features}

- **소프트웨어 증폭기** -- 작은 마이크 소리를 Windows 게인 한계를 넘어 증폭합니다(최대 +50 dB, 잠금 해제 시 +100 dB)
- **노이즈 억제** -- 말하는 중에도 팬 소리, 키보드 소리, 실내 소음을 제거합니다. 경량, 표준(RNNoise), 고품질(DeepFilterNet3) 중에서 선택할 수 있습니다
- **파라메트릭 이퀄라이저** -- 실시간 스펙트럼 그래프를 보며 음색을 조정합니다
- **노이즈 게이트** -- 말과 말 사이에 마이크를 음소거해 완전한 무음을 만듭니다
- **리미터** -- 피크를 상한 아래로 유지해 갑작스러운 큰 소리에도 클리핑이 생기지 않습니다
- **프리셋 및 장치 프로필** -- 사운드 설정과 마이크/출력 조합을 클릭 한 번으로 전환합니다
- **트레이 제어** -- 알림 영역에서 계속 실행되며, 부스트, 프리셋, 장치를 우클릭 한 번으로 바꿀 수 있습니다
- **8개 언어** -- English, 日本語, 简体中文, 繁體中文, 한국어, Deutsch, Français, Español

## 작동 방식 {#how-it-works}

```
마이크
    │
    ▼
캡처 (Windows에서는 WASAPI)
    │
    ▼
노이즈 억제 → 이퀄라이저 → 게인 → 노이즈 게이트 → 리미터
    │
    ▼
가상 오디오 장치 (예: CABLE Input)
    │
    ▼
Discord / Zoom / OBS / Teams / 게임 …
```

## 빠른 시작 {#quick-start}

1. [VB-CABLE](https://vb-audio.com/Cable/)을 설치합니다.
2. 최신 릴리스(설치 프로그램 `...-setup.exe` 또는 포터블 zip)를 다운로드하고 Easy MIC Booster를 실행합니다.
3. **라우팅**에서 **입력**으로 사용하는 마이크를, **출력**으로 `CABLE Input (VB-Audio Virtual Cable)`을 선택합니다.
4. 오른쪽 위의 **BOOST ON**이 켜져 있는지 확인하고 **게인**을 설정합니다.
5. Discord, Zoom, OBS 등에서 마이크로 `CABLE Output (VB-Audio Virtual Cable)`을 선택합니다.

전체 과정은 [사용자 가이드](user-guide)를 참고하세요.
