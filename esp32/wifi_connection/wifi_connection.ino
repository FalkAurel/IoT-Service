#include <Arduino.h>
#include <HTTPClient.h>
#include <WiFi.h>

const char* ssid = "vivo Y72 5G";
const char* password = "test_network";

const char* post_request = R"delim(POST /api/v1/?hi=hjkasd HTTP/1.1
Authorization: Basic dXNlcjI6aV9hbV91c2VyMg==
Uri: api/v1
Content-Type: application/json
Host: 192.168.62.44:3000
Content-Length: 47

{ 
  "temp": 0, 
  "rpm": 0,
  "device_id": 0
})delim";

char server[] = "192.168.62.44"; // Correct IP address

int port = 3000;
WiFiClient client;

void setup() {
    Serial.begin(9600);
    delay(1000);

    WiFi.begin(ssid, password);
    Serial.println("\nConnecting");

    while (WiFi.status() != WL_CONNECTED) {
        Serial.print(".");
        delay(100);
    }

    getNetworkInfo();

    Serial.println("\nConnected to the WiFi network");
    Serial.print("Local ESP32 IP: ");
    Serial.println(WiFi.localIP());

    if (client.connect(server, port)) {
        Serial.println("Connected to Server, sending request...");
        client.print(post_request); // Use client.print instead of println to avoid adding extra newline
        client.println();
        Serial.println("Request sent!");
    } else {
        Serial.println("Couldn't connect to Server");
    }
}

void loop() {}

void getNetworkInfo() {
    if (WiFi.status() == WL_CONNECTED) {
        Serial.print("[*] Network information for ");
        Serial.println(WiFi.SSID());
        Serial.println("[+] BSSID : " + WiFi.BSSIDstr());
        Serial.print("[+] Gateway IP : ");
        Serial.println(WiFi.gatewayIP());
        Serial.print("[+] Subnet Mask : ");
        Serial.println(WiFi.subnetMask());
        Serial.println((String)"[+] RSSI : " + WiFi.RSSI() + " dB");
        Serial.print("[+] ESP32 IP : ");
        Serial.println(WiFi.localIP());
        Serial.println("Other connection address: esp32-1C2514.dhcp.wdf.sap.corp");
    }
}