# ユーザーガイド

Easy MIC Boosterのインストール方法と日々の使い方について説明します。

## 動作環境

- **OS**: Windows 10/11
- **仮想オーディオドライバ**: VB-Audio Virtual Cable (または同等のもの)

## インストール手順

1.  **VB-Cable のインストール**
    *   [vb-audio.com](https://vb-audio.com/Cable/) からダウンロードしてください。
    *   zipファイルを解凍し、`VBCABLE_Setup_x64.exe` を**管理者として実行**してインストールします。
    *   インストール後、PCを再起動してください。

2.  **Easy MIC Booster のダウンロード**
    *   GitHubのリリースページから最新のzipファイルをダウンロードします。
    *   任意のフォルダ（例: `C:\Tools\EasyMICBooster`）に解凍してください。

## 初回セットアップ

1.  **アプリの起動**
    *   解凍したフォルダ内の `EasyMICBooster.exe` を実行します。

2.  **デバイス設定**
    *   **入力デバイス**: 使用する物理マイクを選択します。
    *   **出力デバイス**: 仮想ケーブルの入力（例: "CABLE Input (VB-Audio Virtual Cable)"）を選択します。

3.  **ルーティングの確認**
    *   画面中央の大きなスイッチを **ON** にします。
    *   マイクに向かって話すと、レベルメーターが動くことを確認してください。

4.  **各アプリの設定**
    *   チャットアプリ（Discord, Zoom等）や配信ソフト（OBS）を開きます。
    *   音声設定のマイク（入力デバイス）に「**CABLE Output (VB-Audio Virtual Cable)**」を選択します。

> [!IMPORTANT]
> Windowsのサウンド設定で「CABLE Output」を「既定の再生デバイス」に設定**しない**でください。ハウリング（フィードバックループ）の原因になります。
