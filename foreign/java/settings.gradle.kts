rootProject.name = "iggy-java-client"

include("iggy-java-sdk")
project(":iggy-java-sdk").projectDir = file("java-sdk")

include("iggy-java-example")
project(":iggy-java-example").projectDir = file("examples")

include("iggy-java-example:simple-producer")
project(":iggy-java-example:simple-producer").projectDir = file("examples/simple-producer")

include("iggy-java-example:simple-consumer")
project(":iggy-java-example:simple-consumer").projectDir = file("examples/simple-consumer")
