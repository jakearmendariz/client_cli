# Rust Client Cli tool for Cloudfare Application
Jake Armendariz<br>
jakearmendariz99@gmail.com

## Design
Two operations:
1. `--url <url>` calls a request on any url http or https and prints it's output.  Basically, curl
 - Parse URL into the protocol (HTTP or HTTPS) to find the port (80 or 443), host, and path. 
 - I use a library to connect SSL and then connect with TCP to the host and port
 - Construct an HTTP request and send it across the socket to the host.
 - Wait and read the response, printing to the console
2. `--profile <request_count>` calls requests to my-worker.jakearmendariz.workers.dev and records the speed, bytes, and errors
 - Create a thread pool of size min(request_count, 100) then it tasks each thread a request.
 - I use mutexes and the std::Arc for atomic operations while saving request data between threads.
 - Constructing HTTP requests the same way as before, I slowly record the results
 - A successful response is returned within 1,000ms and a 200 response code. All codes other than 200 are recorded as errors. 
 - Timeouts are marked as unsuccessful in the final result.
 - After completing all requests the response times are sorted and split to find mean, median, min, and max times.
 - Print all of the outputs to the terminal
  - PS: Add `-s` flag to see results from a single-threaded request function

## Result
Here are some of the outputs from the program. I rarely got any HTTP errors. 
The % success is not the percentage of 200's but rather the responses in < 1,000 ms or errors. So the errors occur if the site doesn't respond in time.

Despite my best efforts to break my personal website server, all requests were successful in every domain I tried, receiving only 200 response codes.

Not all websites had the same percent success rate. I expected sites like Google to have a 100% success rate;  however, their success rate was mainly due to the number of bytes being sent. I ranked the website I tested in order of least to most successful, and, as you can see, there is a direct correlation between the number of bytes sent and the number of timeouts. 


### 1,000 requests: 100 threads
```
=====.com - my friend's website
Success: 83.7%
averaging bytes: 121872
min bytes: 0
max bytes: 131827
mean time: 1422.6594982078852
median time: 1173
max time: 6737
min time: 195
Errors: 0

www.google.com
Success: 97.8%
averaging bytes: 49505
min bytes: 0
max bytes: 50388
mean time: 622.0766871165645
median time: 512
max time: 1634
min time: 247
Errors: 0

jakearmendariz.com - my website
Success: 99.2%
averaging bytes: 5682
min bytes: 0
max bytes: 5682
mean time: 55.75775775775776
median time: 27
max time: 942
min time: 10
Errors: 0

my-worker.jakearmendariz.workers.dev
Success: 99.6%
averaging bytes: 2669
min bytes: 0
max bytes: 2925
mean time: 122.22991967871486
median time: 102
max time: 971
min time: 29
Errors: 0

```

## Conclusion
I spent a lot more time on this project than I originally expected. I learned a lot more about rust and its mult-threading features. I am really excited at the possibility of working at Cloudfare and I hope the time I put into this project is evidence of that.

PS: I could improve on this a lot, I am just very tired from school and interviews. But I am a very big fan for project based interviews.

Thank you,
Jake Armendariz