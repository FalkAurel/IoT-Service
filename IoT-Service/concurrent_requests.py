import aiohttp
import asyncio
import time

# Define the request parameters
url = "http://127.0.0.1:3000/api/v1/"
headers = {
    "Authorization": "Basic dXNlcjI6aV9hbV91c2VyMg==",
    "Content-Type": "application/json"
}
params = {
    "device_id": "0",
    "time_start": "22",
    "time_end": "26"
}

# Asynchronous function to send a single GET request
async def send_request(session):
    async with session.get(url, headers=headers, params=params) as response:
        return response.status, await response.text()

# Main function to send 10,000 requests in batches of 500
async def main():
    num_requests = 100000
    batch_size = 500

    start_time = time.time()  # Start the timer

    async with aiohttp.ClientSession() as session:
        for i in range(0, num_requests, batch_size):
            tasks = [send_request(session) for _ in range(batch_size)]
            responses = await asyncio.gather(*tasks)

            for status_code, response_text in responses:
                # Process response if needed
                pass

    end_time = time.time()  # End the timer
    duration = end_time - start_time

    print(f"It took {duration:.2f} seconds to send {num_requests} requests in batches of {batch_size}.")

if __name__ == "__main__":
    asyncio.run(main())
